import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';

// Types matching Rust DTOs
interface SshConnection {
  id: string;
  name: string;
  host: string;
  port: number;
  username: string;
  auth_method: { Password: null } | { PublicKey: { key_path: string } };
}

interface DockerHost {
  id: string;
  name: string;
  url: string;
}

interface CoolifyInstance {
  id: string;
  name: string;
  url: string;
  api_key: string;
}

interface GitRepo {
  id: string;
  name: string;
  path: string;
  remote_url: string | null;
}

interface Config {
  ssh_connections: SshConnection[];
  docker_hosts: DockerHost[];
  coolify_instances: CoolifyInstance[];
  git_repos: GitRepo[];
}

interface Tab {
  id: string;
  label: string;
}

function App() {
  const [activeTab, setActiveTab] = useState('ssh');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Data state
  const [sshConnections, setSshConnections] = useState<SshConnection[]>([]);
  const [dockerHosts, setDockerHosts] = useState<DockerHost[]>([]);
  const [coolifyInstances, setCoolifyInstances] = useState<CoolifyInstance[]>([]);
  const [gitRepos, setGitRepos] = useState<GitRepo[]>([]);

  // Form state
  const [showForm, setShowForm] = useState(false);
  const [formData, setFormData] = useState<Record<string, string>>({});

  const tabs: Tab[] = [
    { id: 'ssh', label: 'SSH Connections' },
    { id: 'docker', label: 'Docker Hosts' },
    { id: 'coolify', label: 'Coolify Instances' },
    { id: 'git', label: 'Git Repositories' },
  ];

  // Load config on mount
  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    try {
      setLoading(true);
      setError(null);
      const config = await invoke<Config>('get_config');
      setSshConnections(config.ssh_connections);
      setDockerHosts(config.docker_hosts);
      setCoolifyInstances(config.coolify_instances);
      setGitRepos(config.git_repos);
    } catch (err) {
      setError(`Failed to load config: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  // SSH handlers
  const addSsh = async () => {
    try {
      await invoke('add_ssh', {
        data: {
          name: formData.name,
          host: formData.host,
          port: formData.port ? parseInt(formData.port) : 22,
          username: formData.username || 'root',
          key_path: formData.key_path || '~/.ssh/id_rsa',
        },
      });
      setShowForm(false);
      setFormData({});
      loadConfig();
    } catch (err) {
      setError(`Failed to add SSH: ${err}`);
    }
  };

  const deleteSsh = async (id: string) => {
    try {
      await invoke('delete_ssh', { id });
      loadConfig();
    } catch (err) {
      setError(`Failed to delete SSH: ${err}`);
    }
  };

  // Docker handlers
  const addDocker = async () => {
    try {
      await invoke('add_docker', {
        data: {
          name: formData.name,
          url: formData.url || 'unix:///var/run/docker.sock',
        },
      });
      setShowForm(false);
      setFormData({});
      loadConfig();
    } catch (err) {
      setError(`Failed to add Docker: ${err}`);
    }
  };

  const deleteDocker = async (id: string) => {
    try {
      await invoke('delete_docker', { id });
      loadConfig();
    } catch (err) {
      setError(`Failed to delete Docker: ${err}`);
    }
  };

  // Coolify handlers
  const addCoolify = async () => {
    try {
      await invoke('add_coolify', {
        data: {
          name: formData.name,
          url: formData.url,
          api_key: formData.api_key,
        },
      });
      setShowForm(false);
      setFormData({});
      loadConfig();
    } catch (err) {
      setError(`Failed to add Coolify: ${err}`);
    }
  };

  const deleteCoolify = async (id: string) => {
    try {
      await invoke('delete_coolify', { id });
      loadConfig();
    } catch (err) {
      setError(`Failed to delete Coolify: ${err}`);
    }
  };

  // Git handlers
  const addGit = async () => {
    try {
      await invoke('add_git', {
        data: {
          name: formData.name,
          path: formData.path,
          remote_url: formData.remote_url || null,
        },
      });
      setShowForm(false);
      setFormData({});
      loadConfig();
    } catch (err) {
      setError(`Failed to add Git: ${err}`);
    }
  };

  const deleteGit = async (id: string) => {
    try {
      await invoke('delete_git', { id });
      loadConfig();
    } catch (err) {
      setError(`Failed to delete Git: ${err}`);
    }
  };

  const handleSubmit = () => {
    switch (activeTab) {
      case 'ssh':
        addSsh();
        break;
      case 'docker':
        addDocker();
        break;
      case 'coolify':
        addCoolify();
        break;
      case 'git':
        addGit();
        break;
    }
  };

  const renderForm = () => {
    const forms: Record<string, JSX.Element> = {
      ssh: (
        <>
          <input
            type="text"
            placeholder="Name"
            value={formData.name || ''}
            onChange={(e) => setFormData({ ...formData, name: e.target.value })}
          />
          <input
            type="text"
            placeholder="Host"
            value={formData.host || ''}
            onChange={(e) => setFormData({ ...formData, host: e.target.value })}
          />
          <input
            type="number"
            placeholder="Port (22)"
            value={formData.port || ''}
            onChange={(e) => setFormData({ ...formData, port: e.target.value })}
          />
          <input
            type="text"
            placeholder="Username (root)"
            value={formData.username || ''}
            onChange={(e) => setFormData({ ...formData, username: e.target.value })}
          />
          <input
            type="text"
            placeholder="Key Path (~/.ssh/id_rsa)"
            value={formData.key_path || ''}
            onChange={(e) => setFormData({ ...formData, key_path: e.target.value })}
          />
        </>
      ),
      docker: (
        <>
          <input
            type="text"
            placeholder="Name"
            value={formData.name || ''}
            onChange={(e) => setFormData({ ...formData, name: e.target.value })}
          />
          <input
            type="text"
            placeholder="URL (unix:///var/run/docker.sock)"
            value={formData.url || ''}
            onChange={(e) => setFormData({ ...formData, url: e.target.value })}
          />
        </>
      ),
      coolify: (
        <>
          <input
            type="text"
            placeholder="Name"
            value={formData.name || ''}
            onChange={(e) => setFormData({ ...formData, name: e.target.value })}
          />
          <input
            type="text"
            placeholder="URL (https://coolify.example.com)"
            value={formData.url || ''}
            onChange={(e) => setFormData({ ...formData, url: e.target.value })}
          />
          <input
            type="password"
            placeholder="API Key"
            value={formData.api_key || ''}
            onChange={(e) => setFormData({ ...formData, api_key: e.target.value })}
          />
        </>
      ),
      git: (
        <>
          <input
            type="text"
            placeholder="Name"
            value={formData.name || ''}
            onChange={(e) => setFormData({ ...formData, name: e.target.value })}
          />
          <input
            type="text"
            placeholder="Path (/path/to/repo)"
            value={formData.path || ''}
            onChange={(e) => setFormData({ ...formData, path: e.target.value })}
          />
          <input
            type="text"
            placeholder="Remote URL (optional)"
            value={formData.remote_url || ''}
            onChange={(e) => setFormData({ ...formData, remote_url: e.target.value })}
          />
        </>
      ),
    };

    return (
      <div className="form-modal">
        <div className="form-content">
          <h3>Add {tabs.find((t) => t.id === activeTab)?.label.slice(0, -1)}</h3>
          <div className="form-fields">{forms[activeTab]}</div>
          <div className="form-actions">
            <button className="btn-secondary" onClick={() => { setShowForm(false); setFormData({}); }}>
              Cancel
            </button>
            <button className="btn-primary" onClick={handleSubmit}>
              Add
            </button>
          </div>
        </div>
      </div>
    );
  };

  const renderList = () => {
    if (loading) {
      return <div className="loading">Loading...</div>;
    }

    const lists: Record<string, JSX.Element> = {
      ssh: sshConnections.length > 0 ? (
        <ul className="item-list">
          {sshConnections.map((conn) => (
            <li key={conn.id} className="item">
              <div className="item-info">
                <span className="item-name">{conn.name}</span>
                <span className="item-detail">
                  {conn.username}@{conn.host}:{conn.port}
                </span>
              </div>
              <button className="btn-delete" onClick={() => deleteSsh(conn.id)}>
                Delete
              </button>
            </li>
          ))}
        </ul>
      ) : (
        <div className="empty-state">
          <p>No SSH connections configured yet.</p>
          <button className="btn-primary" onClick={() => setShowForm(true)}>
            Add Connection
          </button>
        </div>
      ),
      docker: dockerHosts.length > 0 ? (
        <ul className="item-list">
          {dockerHosts.map((host) => (
            <li key={host.id} className="item">
              <div className="item-info">
                <span className="item-name">{host.name}</span>
                <span className="item-detail">{host.url}</span>
              </div>
              <button className="btn-delete" onClick={() => deleteDocker(host.id)}>
                Delete
              </button>
            </li>
          ))}
        </ul>
      ) : (
        <div className="empty-state">
          <p>No Docker hosts configured yet.</p>
          <button className="btn-primary" onClick={() => setShowForm(true)}>
            Add Host
          </button>
        </div>
      ),
      coolify: coolifyInstances.length > 0 ? (
        <ul className="item-list">
          {coolifyInstances.map((instance) => (
            <li key={instance.id} className="item">
              <div className="item-info">
                <span className="item-name">{instance.name}</span>
                <span className="item-detail">{instance.url}</span>
              </div>
              <button className="btn-delete" onClick={() => deleteCoolify(instance.id)}>
                Delete
              </button>
            </li>
          ))}
        </ul>
      ) : (
        <div className="empty-state">
          <p>No Coolify instances configured yet.</p>
          <button className="btn-primary" onClick={() => setShowForm(true)}>
            Add Instance
          </button>
        </div>
      ),
      git: gitRepos.length > 0 ? (
        <ul className="item-list">
          {gitRepos.map((repo) => (
            <li key={repo.id} className="item">
              <div className="item-info">
                <span className="item-name">{repo.name}</span>
                <span className="item-detail">{repo.path}</span>
              </div>
              <button className="btn-delete" onClick={() => deleteGit(repo.id)}>
                Delete
              </button>
            </li>
          ))}
        </ul>
      ) : (
        <div className="empty-state">
          <p>No Git repositories configured yet.</p>
          <button className="btn-primary" onClick={() => setShowForm(true)}>
            Add Repository
          </button>
        </div>
      ),
    };

    return lists[activeTab];
  };

  const hasItems = () => {
    switch (activeTab) {
      case 'ssh': return sshConnections.length > 0;
      case 'docker': return dockerHosts.length > 0;
      case 'coolify': return coolifyInstances.length > 0;
      case 'git': return gitRepos.length > 0;
      default: return false;
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
        {error && <div className="error-banner">{error}</div>}

        <div className="panel">
          <div className="panel-header">
            <div>
              <h2>{tabs.find((t) => t.id === activeTab)?.label}</h2>
              <p className="panel-description">
                {activeTab === 'ssh' && 'Manage your SSH connections to remote servers.'}
                {activeTab === 'docker' && 'Monitor and manage Docker containers across hosts.'}
                {activeTab === 'coolify' && 'Manage deployments on your Coolify instances.'}
                {activeTab === 'git' && 'Create and manage Git releases for your repositories.'}
              </p>
            </div>
            {hasItems() && (
              <button className="btn-primary" onClick={() => setShowForm(true)}>
                Add
              </button>
            )}
          </div>
          {renderList()}
        </div>
      </main>

      {showForm && renderForm()}
    </div>
  );
}

export default App;
