//! Git smart HTTP protocol implementation.
//!
//! Implements the git smart HTTP protocol for fetch and push operations.
//! See: https://git-scm.com/docs/http-protocol

use crate::pack::{PackBuilder, PackParser};
use crate::pktline::{PktLine, PktLineReader, PktLineWriter};
use crate::Result;
use guts_storage::{ObjectId, ObjectStore, Reference, Repository};
use std::io::{Read, Write};

/// Git capabilities we advertise.
const CAPABILITIES: &str =
    "report-status delete-refs side-band-64k quiet ofs-delta agent=guts/0.1.0";

/// A reference advertisement line.
#[derive(Debug, Clone)]
pub struct RefAdvertisement {
    /// Object ID the ref points to.
    pub id: ObjectId,
    /// Reference name.
    pub name: String,
}

/// Advertises references to a client (for fetch/clone).
pub fn advertise_refs<W: Write>(writer: &mut W, repo: &Repository, service: &str) -> Result<()> {
    let mut pkt_writer = PktLineWriter::new(writer);

    // Get all refs
    let refs = repo.refs.list_all();
    let head = repo.refs.resolve_head().ok();

    // First line includes capabilities
    let first_ref = if let Some(head_id) = head {
        format!("{} HEAD\0{}\n", head_id, CAPABILITIES)
    } else if let Some((name, Reference::Direct(id))) = refs.first() {
        format!("{} {}\0{}\n", id, name, CAPABILITIES)
    } else {
        // Empty repo - use zero ID
        let zero_id = "0000000000000000000000000000000000000000";
        format!("{} capabilities^{{}}\0{}\n", zero_id, CAPABILITIES)
    };

    pkt_writer.write(&PktLine::from_string(&format!("# service={}\n", service)))?;
    pkt_writer.flush_pkt()?;

    pkt_writer.write(&PktLine::from_string(&first_ref))?;

    // Write remaining refs
    for (name, reference) in &refs {
        if let Reference::Direct(id) = reference {
            pkt_writer.write_line(&format!("{} {}", id, name))?;
        }
    }

    pkt_writer.flush_pkt()?;
    pkt_writer.flush()?;

    Ok(())
}

/// Want/Have negotiation for upload-pack.
#[derive(Debug, Clone)]
pub struct WantHave {
    /// Object IDs the client wants.
    pub wants: Vec<ObjectId>,
    /// Object IDs the client has (for delta compression).
    pub haves: Vec<ObjectId>,
}

impl WantHave {
    /// Parses want/have lines from the client.
    pub fn parse<R: Read>(reader: &mut R) -> Result<Self> {
        let mut pkt_reader = PktLineReader::new(reader);
        let mut wants = Vec::new();
        let mut haves = Vec::new();

        // Read wants
        loop {
            match pkt_reader.read()? {
                Some(PktLine::Data(data)) => {
                    let line = String::from_utf8_lossy(&data);
                    let line = line.trim();

                    if line.starts_with("want ") {
                        let id_str = &line[5..45];
                        wants.push(ObjectId::from_hex(id_str)?);
                    } else if line.starts_with("have ") {
                        let id_str = &line[5..45];
                        haves.push(ObjectId::from_hex(id_str)?);
                    } else if line == "done" {
                        break;
                    }
                }
                Some(PktLine::Flush) => {
                    // After wants, read haves
                    continue;
                }
                _ => break,
            }
        }

        Ok(Self { wants, haves })
    }
}

/// Handles git-upload-pack (fetch/clone).
pub fn upload_pack<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    repo: &Repository,
) -> Result<()> {
    let want_have = WantHave::parse(reader)?;
    let mut pkt_writer = PktLineWriter::new(writer);

    if want_have.wants.is_empty() {
        pkt_writer.write_line("NAK")?;
        pkt_writer.flush()?;
        return Ok(());
    }

    // Build pack with requested objects
    let mut builder = PackBuilder::new();

    for want_id in &want_have.wants {
        // Add the wanted object and its dependencies
        collect_objects(&repo.objects, want_id, &want_have.haves, &mut builder)?;
    }

    let pack = builder.build()?;

    // Send ACK and pack data
    pkt_writer.write_line("NAK")?; // We don't do common ancestor negotiation yet

    // Send pack via side-band
    // Side-band channel 1 is for pack data
    for chunk in pack.chunks(65515) {
        // Max side-band payload
        let mut data = vec![1u8]; // Channel 1
        data.extend_from_slice(chunk);
        pkt_writer.write(&PktLine::Data(data))?;
    }

    pkt_writer.flush_pkt()?;
    pkt_writer.flush()?;

    Ok(())
}

/// Collects an object and its dependencies for packing.
fn collect_objects(
    store: &ObjectStore,
    id: &ObjectId,
    have: &[ObjectId],
    builder: &mut PackBuilder,
) -> Result<()> {
    // Skip if client already has this object
    if have.contains(id) {
        return Ok(());
    }

    if let Ok(object) = store.get(id) {
        builder.add(object.clone());

        // For commits, also collect tree and parents
        if object.object_type == guts_storage::ObjectType::Commit {
            // Parse commit to find tree and parents
            let content = String::from_utf8_lossy(&object.data);
            for line in content.lines() {
                if let Some(tree_hex) = line.strip_prefix("tree ") {
                    if let Ok(tree_id) = ObjectId::from_hex(tree_hex) {
                        collect_objects(store, &tree_id, have, builder)?;
                    }
                } else if let Some(parent_hex) = line.strip_prefix("parent ") {
                    if let Ok(parent_id) = ObjectId::from_hex(parent_hex) {
                        collect_objects(store, &parent_id, have, builder)?;
                    }
                } else if line.is_empty() {
                    break; // End of headers
                }
            }
        }
        // For trees, collect blobs and subtrees
        else if object.object_type == guts_storage::ObjectType::Tree {
            // Parse tree entries (simplified - real git uses binary format)
            // For MVP, we'll handle this when we implement proper tree serialization
        }
    }

    Ok(())
}

/// A ref update command from the client.
#[derive(Debug, Clone)]
pub struct Command {
    /// Old object ID (zeros for create).
    pub old_id: ObjectId,
    /// New object ID (zeros for delete).
    pub new_id: ObjectId,
    /// Reference name.
    pub ref_name: String,
}

impl Command {
    /// Checks if this is a create command.
    pub fn is_create(&self) -> bool {
        self.old_id.to_hex() == "0000000000000000000000000000000000000000"
    }

    /// Checks if this is a delete command.
    pub fn is_delete(&self) -> bool {
        self.new_id.to_hex() == "0000000000000000000000000000000000000000"
    }
}

/// Handles git-receive-pack (push).
pub fn receive_pack<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    repo: &Repository,
) -> Result<Vec<Command>> {
    let mut pkt_reader = PktLineReader::new(reader);
    let mut commands = Vec::new();

    // Read commands
    loop {
        match pkt_reader.read()? {
            Some(PktLine::Data(data)) => {
                let line = String::from_utf8_lossy(&data);
                let line = line.trim();

                // Parse: old-id new-id ref-name
                let parts: Vec<&str> = line.splitn(3, ' ').collect();
                if parts.len() >= 3 {
                    let old_id = ObjectId::from_hex(parts[0])?;
                    let new_id = ObjectId::from_hex(parts[1])?;
                    let ref_name = parts[2].split('\0').next().unwrap_or(parts[2]).to_string();

                    commands.push(Command {
                        old_id,
                        new_id,
                        ref_name,
                    });
                }
            }
            Some(PktLine::Flush) | None => break,
            _ => continue,
        }
    }

    // Read pack data
    let mut pack_data = Vec::new();
    // Read remaining data directly from the underlying reader
    pkt_reader.inner_mut().read_to_end(&mut pack_data)?;

    if !pack_data.is_empty() {
        // Parse pack and store objects
        let mut parser = PackParser::new(&pack_data);
        parser.parse(&repo.objects)?;
    }

    // Apply ref updates
    for cmd in &commands {
        if cmd.is_delete() {
            let _ = repo.refs.delete(&cmd.ref_name);
        } else {
            repo.refs.set(&cmd.ref_name, cmd.new_id);
        }
    }

    // Send status report
    let mut pkt_writer = PktLineWriter::new(writer);
    pkt_writer.write_line("unpack ok")?;
    for cmd in &commands {
        pkt_writer.write_line(&format!("ok {}", cmd.ref_name))?;
    }
    pkt_writer.flush_pkt()?;
    pkt_writer.flush()?;

    Ok(commands)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ref_advertisement() {
        let repo = Repository::new("test", "alice");

        // Add an object and ref
        let blob = guts_storage::GitObject::blob(b"test".to_vec());
        let id = repo.objects.put(blob);
        repo.refs.set("refs/heads/main", id);

        let mut output = Vec::new();
        advertise_refs(&mut output, &repo, "git-upload-pack").unwrap();

        let output_str = String::from_utf8_lossy(&output);
        assert!(output_str.contains("git-upload-pack"));
        assert!(output_str.contains(&id.to_hex()));
    }
}
