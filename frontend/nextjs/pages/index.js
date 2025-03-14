import React from 'react';
import Head from 'next/head';
import Link from 'next/link';

export default function Home() {
  return (
    <div className="container mx-auto px-4 py-8">
      <Head>
        <title>ELXR Chain | Kombucha Platform</title>
        <meta name="description" content="Decentralized kombucha tracking and trading platform" />
        <link rel="icon" href="/favicon.ico" />
      </Head>

      <header className="mb-10">
        <h1 className="text-4xl font-bold text-green-800">ELXR Chain</h1>
        <p className="text-xl text-gray-600">Quantum-Secure Kombucha Platform</p>
      </header>

      <main>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-8 mb-12">
          <div className="bg-white rounded-lg shadow-lg p-6 border-t-4 border-green-500">
            <h2 className="text-2xl font-semibold mb-4">Personal Kombucha Dashboard</h2>
            <p className="mb-4">Monitor your personal kombucha fermentation in real-time with quantum-secured telemetry.</p>
            <Link href="/dashboard">
              <a className="px-4 py-2 bg-green-500 text-white rounded hover:bg-green-600 transition">View Dashboard</a>
            </Link>
          </div>

          <div className="bg-white rounded-lg shadow-lg p-6 border-t-4 border-blue-500">
            <h2 className="text-2xl font-semibold mb-4">ELXR/NRSH DEX</h2>
            <p className="mb-4">Trade ELXR and NRSH tokens with quantum-resistant security and low fees.</p>
            <Link href="/dex">
              <a className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 transition">Open DEX</a>
            </Link>
          </div>
        </div>

        <div className="bg-white rounded-lg shadow-lg p-6 border-t-4 border-purple-500 mb-12">
          <h2 className="text-2xl font-semibold mb-4">Kombucha Social Hub</h2>
          <p className="mb-4">Connect with other kombucha brewers, share recipes, and trade cultures.</p>
          <Link href="/social">
            <a className="px-4 py-2 bg-purple-500 text-white rounded hover:bg-purple-600 transition">Join Community</a>
          </Link>
        </div>
      </main>

      <footer className="mt-10 pt-10 border-t border-gray-200 text-center text-gray-500">
        <p>Powered by Matrix-Magiq | ELXR Chain</p>
      </footer>
    </div>
  );
}
