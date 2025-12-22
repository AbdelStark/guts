import { mockRepositories } from "@/data/mock";
import RepositoryContent from "./RepositoryContent";

// Generate static params for all mock repositories (required for static export)
export function generateStaticParams() {
  return mockRepositories.map((repo) => ({
    owner: repo.owner,
    repo: repo.name,
  }));
}

export default function RepositoryPage() {
  return <RepositoryContent />;
}
