import type { Metadata } from 'next';
import '../styles/globals.css';

export const metadata: Metadata = {
  title: 'Sentinelle-MCP - Votre Garde du Corps Numérique',
  description: 'Plateforme de sécurité managée avec IA pour TPE/PME',
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="fr">
      <body>{children}</body>
    </html>
  );
}
