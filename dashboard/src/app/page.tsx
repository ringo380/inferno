export default function HomePage() {
  return (
    <div className="min-h-screen bg-gray-900 text-white p-8">
      <h1 className="text-3xl font-bold mb-4">Inferno Dashboard</h1>
      <p className="text-gray-300">Enterprise AI/ML Model Management Platform</p>
      <div className="mt-8 p-4 bg-gray-800 rounded-lg">
        <h2 className="text-xl font-semibold mb-2">Test Status</h2>
        <p className="text-green-400">✓ Application is loading successfully</p>
        <p className="text-green-400">✓ React components are rendering</p>
        <p className="text-green-400">✓ Dark theme is applied</p>
      </div>
    </div>
  );
}