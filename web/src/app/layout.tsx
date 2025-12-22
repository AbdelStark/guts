import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "GUTS - Decentralized Code Collaboration",
  description: "GitHub Uncaptured Trustless Sovereign - A censorship-resistant code collaboration platform",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
