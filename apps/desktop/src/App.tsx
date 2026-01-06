import { useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';

interface Tab {
  id: string;
  label: string;
}

function App() {
  const [activeTab, setActiveTab] = useState('ssh');
  const [message, setMessage] = useState('');

  const tabs: Tab[] = [
    { id: 'ssh', label: 'SSH Connections' },
    { id: 'docker', label: 'Docker Containers' },
    { id: 'coolify', label: 'Coolify Deployments' },
    { id: 'git', label: 'Git Releases' },
  ];

  const handleTest = async () => {
    try {
      const result = await invoke('greet', { name: 'pctrl' });
      setMessage(result as string);
    } catch (error) {
      setMessage(`Error: ${error}`);
    }
  };

  return (
    <div className="container">
      <header>
        <h1>pctrl</h1>
        <p>Mission Control for Self-Hosters & Indie Devs</p>
      </header>

      <nav className="tabs">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            className={`tab ${activeTab === tab.id ? 'active' : ''}`}
            onClick={() => setActiveTab(tab.id)}
          >
            {tab.label}
          </button>
        ))}
      </nav>

      <main className="content">
        {activeTab === 'ssh' && (
          <div className="panel">
            <h2>SSH Connections</h2>
            <p>Manage your SSH connections to remote servers.</p>
            <div className="empty-state">
              <p>No SSH connections configured yet.</p>
              <button className="btn-primary">Add Connection</button>
            </div>
          </div>
        )}

        {activeTab === 'docker' && (
          <div className="panel">
            <h2>Docker Containers</h2>
            <p>Monitor and manage Docker containers across hosts.</p>
            <div className="empty-state">
              <p>No Docker hosts configured yet.</p>
              <button className="btn-primary">Add Host</button>
            </div>
          </div>
        )}

        {activeTab === 'coolify' && (
          <div className="panel">
            <h2>Coolify Deployments</h2>
            <p>Manage deployments on your Coolify instances.</p>
            <div className="empty-state">
              <p>No Coolify instances configured yet.</p>
              <button className="btn-primary">Add Instance</button>
            </div>
          </div>
        )}

        {activeTab === 'git' && (
          <div className="panel">
            <h2>Git Releases</h2>
            <p>Create and manage Git releases for your repositories.</p>
            <div className="empty-state">
              <p>No Git repositories configured yet.</p>
              <button className="btn-primary">Add Repository</button>
            </div>
          </div>
        )}
      </main>

      <footer>
        <button onClick={handleTest}>Test Backend</button>
        {message && <p>{message}</p>}
      </footer>
    </div>
  );
}

export default App;
