//! End-to-end collaboration test.
//!
//! This test verifies that two clients can collaborate on a repository:
//! 1. Client 1 creates a repository and pushes content
//! 2. Client 2 clones the repository, makes changes, and pushes
//! 3. Client 1 pulls and sees Client 2's changes
//!
//! For the MVP, we test with a single node. Multi-node replication
//! will be added in a future iteration.

use std::io::Cursor;

use guts_git::{advertise_refs, receive_pack, PackBuilder, PktLine, PktLineWriter};
use guts_storage::{GitObject, ObjectId, ObjectStore, ObjectType, Repository};

/// Test helper to create a simple tree object.
fn create_tree(store: &ObjectStore, entries: &[(&str, ObjectId)]) -> ObjectId {
    // Git tree format: mode SP path NUL sha1
    let mut data = Vec::new();
    for (name, id) in entries {
        data.extend_from_slice(b"100644 ");
        data.extend_from_slice(name.as_bytes());
        data.push(0);
        data.extend_from_slice(id.as_bytes());
    }
    let tree = GitObject::new(ObjectType::Tree, data);
    store.put(tree)
}

/// Creates a commit object and returns its ID.
#[allow(dead_code)]
fn create_commit(
    store: &ObjectStore,
    tree_id: ObjectId,
    parents: &[ObjectId],
    message: &str,
    author: &str,
) -> ObjectId {
    let commit = GitObject::commit(&tree_id, parents, author, author, message);
    store.put(commit)
}

/// Simulates a git push by sending pack data to the repository.
fn simulate_push(repo: &Repository, objects: &[GitObject], ref_name: &str, new_id: ObjectId) {
    // Build pack with objects
    let mut builder = PackBuilder::default();
    for obj in objects {
        builder.add(obj.clone());
    }
    let pack = builder.build().unwrap();

    // Create push request
    let old_id = ObjectId::from_hex("0000000000000000000000000000000000000000").unwrap();
    let cmd_line = format!(
        "{} {} {}\0report-status\n",
        old_id.to_hex(),
        new_id.to_hex(),
        ref_name
    );

    let mut request = Vec::new();
    {
        let mut writer = PktLineWriter::new(&mut request);
        writer.write(&PktLine::Data(cmd_line.into_bytes())).unwrap();
        writer.flush_pkt().unwrap();
    }
    request.extend_from_slice(&pack);

    // Execute push
    let mut input = Cursor::new(request);
    let mut output = Vec::new();
    receive_pack(&mut input, &mut output, repo).unwrap();

    // Verify success
    let output_str = String::from_utf8_lossy(&output);
    assert!(
        output_str.contains("unpack ok"),
        "Push failed: {}",
        output_str
    );
}

#[test]
fn test_single_node_collaboration() {
    // Create a repository (simulates the Guts node)
    let repo = Repository::new("test-repo", "alice");

    // =========================================
    // Client 1: Create initial content and push
    // =========================================

    // Create a file
    let file1_content = b"Hello from Client 1!\n";
    let blob1 = GitObject::blob(file1_content.to_vec());
    let blob1_id = blob1.id;

    // Create tree with the file
    let tree1_id = create_tree(&repo.objects, &[("README.md", blob1_id)]);

    // Create initial commit
    let author1 = "Client1 <client1@example.com> 1700000000 +0000";
    let commit1 = GitObject::commit(
        &tree1_id,
        &[],
        author1,
        author1,
        "Initial commit from Client 1",
    );
    let commit1_id = commit1.id;

    // Push to repository
    simulate_push(
        &repo,
        &[
            blob1.clone(),
            GitObject::new(ObjectType::Tree, {
                let mut data = Vec::new();
                data.extend_from_slice(b"100644 ");
                data.extend_from_slice(b"README.md");
                data.push(0);
                data.extend_from_slice(blob1_id.as_bytes());
                data
            }),
            commit1.clone(),
        ],
        "refs/heads/main",
        commit1_id,
    );

    // Verify repository state
    assert!(repo.objects.contains(&commit1_id));
    let main_ref = repo.refs.get("refs/heads/main").unwrap();
    assert_eq!(main_ref.resolve(&repo.refs).unwrap(), commit1_id);

    println!("Client 1 pushed: {}", commit1_id);

    // =========================================
    // Client 2: Clone, modify, and push
    // =========================================

    // Simulate clone by reading refs
    let refs = repo.refs.list_all();
    assert!(!refs.is_empty(), "Repository should have refs after push");

    // Create new content
    let file2_content = b"Hello from Client 2!\n";
    let blob2 = GitObject::blob(file2_content.to_vec());
    let blob2_id = blob2.id;

    // Create new tree with both files
    let tree2_id = create_tree(
        &repo.objects,
        &[("README.md", blob1_id), ("client2.txt", blob2_id)],
    );

    // Create commit with parent
    let author2 = "Client2 <client2@example.com> 1700001000 +0000";
    let commit2 = GitObject::commit(
        &tree2_id,
        &[commit1_id],
        author2,
        author2,
        "Add file from Client 2",
    );
    let commit2_id = commit2.id;

    // Push Client 2's changes
    simulate_push(
        &repo,
        &[
            blob2.clone(),
            GitObject::new(ObjectType::Tree, {
                let mut data = Vec::new();
                // Entry 1: README.md
                data.extend_from_slice(b"100644 ");
                data.extend_from_slice(b"README.md");
                data.push(0);
                data.extend_from_slice(blob1_id.as_bytes());
                // Entry 2: client2.txt
                data.extend_from_slice(b"100644 ");
                data.extend_from_slice(b"client2.txt");
                data.push(0);
                data.extend_from_slice(blob2_id.as_bytes());
                data
            }),
            commit2.clone(),
        ],
        "refs/heads/main",
        commit2_id,
    );

    // Verify repository state
    assert!(repo.objects.contains(&commit2_id));
    let main_ref = repo.refs.get("refs/heads/main").unwrap();
    assert_eq!(main_ref.resolve(&repo.refs).unwrap(), commit2_id);

    println!("Client 2 pushed: {}", commit2_id);

    // =========================================
    // Client 1: Pull and verify
    // =========================================

    // Verify Client 1 can see Client 2's commit
    let latest = repo.refs.resolve_head().unwrap();
    assert_eq!(latest, commit2_id);

    // Verify commit chain
    let commit2_obj = repo.objects.get(&commit2_id).unwrap();
    let commit2_content = String::from_utf8_lossy(&commit2_obj.data);
    assert!(commit2_content.contains(&format!("parent {}", commit1_id)));

    // Verify both blobs exist
    assert!(repo.objects.contains(&blob1_id));
    assert!(repo.objects.contains(&blob2_id));

    println!("Collaboration test passed!");
    println!("  - Client 1 commit: {}", commit1_id);
    println!("  - Client 2 commit: {}", commit2_id);
    println!("  - Total objects: {}", repo.objects.len());
}

#[test]
fn test_ref_advertisement() {
    let repo = Repository::new("test-repo", "owner");

    // Create a commit
    let blob = GitObject::blob(b"test content".to_vec());
    let blob_id = repo.objects.put(blob);

    let tree_id = create_tree(&repo.objects, &[("file.txt", blob_id)]);

    let author = "Test <test@example.com> 1700000000 +0000";
    let commit = GitObject::commit(&tree_id, &[], author, author, "Test commit");
    let commit_id = repo.objects.put(commit);

    repo.refs.set("refs/heads/main", commit_id);

    // Get ref advertisement
    let mut output = Vec::new();
    advertise_refs(&mut output, &repo, "git-upload-pack").unwrap();

    let output_str = String::from_utf8_lossy(&output);

    // Verify it contains the service header
    assert!(output_str.contains("git-upload-pack"));

    // Verify it contains the commit ID
    assert!(output_str.contains(&commit_id.to_hex()));

    // Verify it advertises capabilities
    assert!(output_str.contains("report-status"));
}

#[test]
fn test_multiple_branches() {
    let repo = Repository::new("test-repo", "owner");

    // Create content
    let blob = GitObject::blob(b"content".to_vec());
    let blob_id = repo.objects.put(blob);
    let tree_id = create_tree(&repo.objects, &[("file.txt", blob_id)]);

    // Create main branch commit
    let author = "Test <test@example.com> 1700000000 +0000";
    let main_commit = GitObject::commit(&tree_id, &[], author, author, "Main commit");
    let main_id = repo.objects.put(main_commit);
    repo.refs.set("refs/heads/main", main_id);

    // Create feature branch commit
    let blob2 = GitObject::blob(b"feature content".to_vec());
    let blob2_id = repo.objects.put(blob2);
    let tree2_id = create_tree(
        &repo.objects,
        &[("file.txt", blob_id), ("feature.txt", blob2_id)],
    );

    let feature_commit = GitObject::commit(&tree2_id, &[main_id], author, author, "Feature commit");
    let feature_id = repo.objects.put(feature_commit);
    repo.refs.set("refs/heads/feature", feature_id);

    // Verify both branches exist
    let refs = repo.refs.list("refs/heads/");
    assert_eq!(refs.len(), 2);

    let main_ref = repo.refs.get("refs/heads/main").unwrap();
    assert_eq!(main_ref.resolve(&repo.refs).unwrap(), main_id);

    let feature_ref = repo.refs.get("refs/heads/feature").unwrap();
    assert_eq!(feature_ref.resolve(&repo.refs).unwrap(), feature_id);
}
