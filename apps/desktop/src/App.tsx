import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';

// ─────────────────────────────────────────────────────────────────────────────
// v6 Types
// ─────────────────────────────────────────────────────────────────────────────

interface Project {
  id: string;
  name: string;
  description: string | null;
  stack: string[];
  status: string;
}

interface Server {
  id: string;
  name: string;
  host: string;
  server_type: string;
  provider: string | null;
}

interface Domain {
  id: string;
  domain: string;
  domain_type: string;
  ssl: boolean;
}

interface DatabaseCredentials {
  id: string;
  name: string;
  db_type: string;
  host: string | null;
  port: number | null;
  username: string | null;
}

interface Script {
  id: string;
  name: string;
  command: string;
  script_type: string;
  description: string | null;
}

interface Tab {
  id: string;
  label: string;
  color: string;
}

// ─────────────────────────────────────────────────────────────────────────────
// App Component
// ─────────────────────────────────────────────────────────────────────────────

function App() {
  const [activeTab, setActiveTab] = useState('projects');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // v6 Data state
  const [projects, setProjects] = useState<Project[]>([]);
  const [servers, setServers] = useState<Server[]>([]);
  const [domains, setDomains] = useState<Domain[]>([]);
  const [databases, setDatabases] = useState<DatabaseCredentials[]>([]);
  const [scripts, setScripts] = useState<Script[]>([]);

  // Form state
  const [showForm, setShowForm] = useState(false);
  const [formData, setFormData] = useState<Record<string, string>>({});

  const tabs: Tab[] = [
    { id: 'projects', label: 'Projects', color: '#00bcd4' },
    { id: 'servers', label: 'Servers', color: '#4caf50' },
    { id: 'domains', label: 'Domains', color: '#2196f3' },
    { id: 'databases', label: 'Databases', color: '#9c27b0' },
    { id: 'scripts', label: 'Scripts', color: '#ff9800' },
  ];

  // Load data on mount
  useEffect(() => {
    loadAllData();
  }, []);

  const loadAllData = async () => {
    try {
      setLoading(true);
      setError(null);

      const [projectsData, serversData, domainsData, databasesData, scriptsData] =
        await Promise.all([
          invoke<Project[]>('list_projects'),
          invoke<Server[]>('list_servers'),
          invoke<Domain[]>('list_domains'),
          invoke<DatabaseCredentials[]>('list_databases'),
          invoke<Script[]>('list_scripts'),
        ]);

      setProjects(projectsData);
      setServers(serversData);
      setDomains(domainsData);
      setDatabases(databasesData);
      setScripts(scriptsData);
    } catch (err) {
      setError(`Failed to load data: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  // ─────────────────────────────────────────────────────────────────────────────
  // Add Handlers
  // ─────────────────────────────────────────────────────────────────────────────

  const addProject = async () => {
    try {
      await invoke('add_project', {
        data: {
          name: formData.name,
          description: formData.description || null,
          stack: formData.stack ? formData.stack.split(',').map((s) => s.trim()) : [],
          status: formData.status || 'dev',
        },
      });
      closeForm();
      loadAllData();
    } catch (err) {
      setError(`Failed to add project: ${err}`);
    }
  };

  const addServer = async () => {
    try {
      await invoke('add_server', {
        data: {
          name: formData.name,
          host: formData.host,
          server_type: formData.server_type || 'vps',
          provider: formData.provider || null,
        },
      });
      closeForm();
      loadAllData();
    } catch (err) {
      setError(`Failed to add server: ${err}`);
    }
  };

  const addDomain = async () => {
    try {
      await invoke('add_domain', {
        data: {
          domain: formData.domain,
          domain_type: formData.domain_type || 'production',
          ssl: formData.ssl !== 'false',
        },
      });
      closeForm();
      loadAllData();
    } catch (err) {
      setError(`Failed to add domain: ${err}`);
    }
  };

  const addDatabase = async () => {
    try {
      await invoke('add_database', {
        data: {
          name: formData.name,
          db_type: formData.db_type || 'postgres',
          host: formData.host || null,
          port: formData.port ? parseInt(formData.port) : null,
          username: formData.username || null,
          password: formData.password || null,
        },
      });
      closeForm();
      loadAllData();
    } catch (err) {
      setError(`Failed to add database: ${err}`);
    }
  };

  const addScript = async () => {
    try {
      await invoke('add_script', {
        data: {
          name: formData.name,
          command: formData.command,
          script_type: formData.script_type || 'local',
          description: formData.description || null,
        },
      });
      closeForm();
      loadAllData();
    } catch (err) {
      setError(`Failed to add script: ${err}`);
    }
  };

  // ─────────────────────────────────────────────────────────────────────────────
  // Delete Handlers
  // ─────────────────────────────────────────────────────────────────────────────

  const deleteProject = async (id: string) => {
    try {
      await invoke('delete_project', { id });
      loadAllData();
    } catch (err) {
      setError(`Failed to delete: ${err}`);
    }
  };

  const deleteServer = async (id: string) => {
    try {
      await invoke('delete_server', { id });
      loadAllData();
    } catch (err) {
      setError(`Failed to delete: ${err}`);
    }
  };

  const deleteDomain = async (id: string) => {
    try {
      await invoke('delete_domain', { id });
      loadAllData();
    } catch (err) {
      setError(`Failed to delete: ${err}`);
    }
  };

  const deleteDatabase = async (id: string) => {
    try {
      await invoke('delete_database', { id });
      loadAllData();
    } catch (err) {
      setError(`Failed to delete: ${err}`);
    }
  };

  const deleteScript = async (id: string) => {
    try {
      await invoke('delete_script', { id });
      loadAllData();
    } catch (err) {
      setError(`Failed to delete: ${err}`);
    }
  };

  // ─────────────────────────────────────────────────────────────────────────────
  // Form Helpers
  // ─────────────────────────────────────────────────────────────────────────────

  const closeForm = () => {
    setShowForm(false);
    setFormData({});
  };

  const handleSubmit = () => {
    switch (activeTab) {
      case 'projects':
        addProject();
        break;
      case 'servers':
        addServer();
        break;
      case 'domains':
        addDomain();
        break;
      case 'databases':
        addDatabase();
        break;
      case 'scripts':
        addScript();
        break;
    }
  };

  const getStatusColor = (status: string) => {
    switch (status.toLowerCase()) {
      case 'dev':
        return '#ff9800';
      case 'staging':
        return '#2196f3';
      case 'live':
        return '#4caf50';
      case 'archived':
        return '#666';
      default:
        return '#999';
    }
  };

  // ─────────────────────────────────────────────────────────────────────────────
  // Render Forms
  // ─────────────────────────────────────────────────────────────────────────────

  const renderForm = () => {
    const forms: Record<string, JSX.Element> = {
      projects: (
        <>
          <input
            type="text"
            placeholder="Name *"
            value={formData.name || ''}
            onChange={(e) => setFormData({ ...formData, name: e.target.value })}
          />
          <input
            type="text"
            placeholder="Description"
            value={formData.description || ''}
            onChange={(e) => setFormData({ ...formData, description: e.target.value })}
          />
          <input
            type="text"
            placeholder="Stack (comma-separated)"
            value={formData.stack || ''}
            onChange={(e) => setFormData({ ...formData, stack: e.target.value })}
          />
          <select
            value={formData.status || 'dev'}
            onChange={(e) => setFormData({ ...formData, status: e.target.value })}
          >
            <option value="dev">Dev</option>
            <option value="staging">Staging</option>
            <option value="live">Live</option>
            <option value="archived">Archived</option>
          </select>
        </>
      ),
      servers: (
        <>
          <input
            type="text"
            placeholder="Name *"
            value={formData.name || ''}
            onChange={(e) => setFormData({ ...formData, name: e.target.value })}
          />
          <input
            type="text"
            placeholder="Host *"
            value={formData.host || ''}
            onChange={(e) => setFormData({ ...formData, host: e.target.value })}
          />
          <select
            value={formData.server_type || 'vps'}
            onChange={(e) => setFormData({ ...formData, server_type: e.target.value })}
          >
            <option value="vps">VPS</option>
            <option value="dedicated">Dedicated</option>
            <option value="local">Local</option>
            <option value="cloud">Cloud</option>
          </select>
          <input
            type="text"
            placeholder="Provider (optional)"
            value={formData.provider || ''}
            onChange={(e) => setFormData({ ...formData, provider: e.target.value })}
          />
        </>
      ),
      domains: (
        <>
          <input
            type="text"
            placeholder="Domain *"
            value={formData.domain || ''}
            onChange={(e) => setFormData({ ...formData, domain: e.target.value })}
          />
          <select
            value={formData.domain_type || 'production'}
            onChange={(e) => setFormData({ ...formData, domain_type: e.target.value })}
          >
            <option value="production">Production</option>
            <option value="staging">Staging</option>
            <option value="dev">Dev</option>
          </select>
          <select
            value={formData.ssl || 'true'}
            onChange={(e) => setFormData({ ...formData, ssl: e.target.value })}
          >
            <option value="true">SSL Enabled</option>
            <option value="false">SSL Disabled</option>
          </select>
        </>
      ),
      databases: (
        <>
          <input
            type="text"
            placeholder="Name *"
            value={formData.name || ''}
            onChange={(e) => setFormData({ ...formData, name: e.target.value })}
          />
          <select
            value={formData.db_type || 'postgres'}
            onChange={(e) => setFormData({ ...formData, db_type: e.target.value })}
          >
            <option value="postgres">PostgreSQL</option>
            <option value="mysql">MySQL</option>
            <option value="mongodb">MongoDB</option>
            <option value="redis">Redis</option>
            <option value="sqlite">SQLite</option>
          </select>
          <input
            type="text"
            placeholder="Host"
            value={formData.host || ''}
            onChange={(e) => setFormData({ ...formData, host: e.target.value })}
          />
          <input
            type="number"
            placeholder="Port"
            value={formData.port || ''}
            onChange={(e) => setFormData({ ...formData, port: e.target.value })}
          />
        </>
      ),
      scripts: (
        <>
          <input
            type="text"
            placeholder="Name *"
            value={formData.name || ''}
            onChange={(e) => setFormData({ ...formData, name: e.target.value })}
          />
          <input
            type="text"
            placeholder="Command *"
            value={formData.command || ''}
            onChange={(e) => setFormData({ ...formData, command: e.target.value })}
          />
          <select
            value={formData.script_type || 'local'}
            onChange={(e) => setFormData({ ...formData, script_type: e.target.value })}
          >
            <option value="local">Local</option>
            <option value="ssh">SSH</option>
            <option value="docker">Docker</option>
          </select>
        </>
      ),
    };

    return (
      <div className="form-modal">
        <div className="form-content">
          <h3>Add {tabs.find((t) => t.id === activeTab)?.label.slice(0, -1)}</h3>
          <div className="form-fields">{forms[activeTab]}</div>
          <div className="form-actions">
            <button className="btn-secondary" onClick={closeForm}>
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

  // ─────────────────────────────────────────────────────────────────────────────
  // Render Lists
  // ─────────────────────────────────────────────────────────────────────────────

  const renderList = () => {
    if (loading) {
      return <div className="loading">Loading...</div>;
    }

    const lists: Record<string, JSX.Element> = {
      projects:
        projects.length > 0 ? (
          <ul className="item-list">
            {projects.map((item) => (
              <li key={item.id} className="item">
                <div className="item-info">
                  <span className="item-status" style={{ backgroundColor: getStatusColor(item.status) }} />
                  <span className="item-name">{item.name}</span>
                  <span className="item-badge" style={{ backgroundColor: getStatusColor(item.status) }}>
                    {item.status}
                  </span>
                  {item.stack.length > 0 && (
                    <span className="item-detail">[{item.stack.join(', ')}]</span>
                  )}
                </div>
                <button className="btn-delete" onClick={() => deleteProject(item.id)}>
                  Delete
                </button>
              </li>
            ))}
          </ul>
        ) : (
          <EmptyState tab="projects" onAdd={() => setShowForm(true)} />
        ),
      servers:
        servers.length > 0 ? (
          <ul className="item-list">
            {servers.map((item) => (
              <li key={item.id} className="item">
                <div className="item-info">
                  <span className="item-status" style={{ backgroundColor: '#4caf50' }} />
                  <span className="item-name">{item.name}</span>
                  <span className="item-detail">{item.host}</span>
                  <span className="item-badge">{item.server_type}</span>
                </div>
                <button className="btn-delete" onClick={() => deleteServer(item.id)}>
                  Delete
                </button>
              </li>
            ))}
          </ul>
        ) : (
          <EmptyState tab="servers" onAdd={() => setShowForm(true)} />
        ),
      domains:
        domains.length > 0 ? (
          <ul className="item-list">
            {domains.map((item) => (
              <li key={item.id} className="item">
                <div className="item-info">
                  <span className="item-ssl">{item.ssl ? '🔒' : '🔓'}</span>
                  <span className="item-name">{item.domain}</span>
                  <span className="item-badge">{item.domain_type}</span>
                </div>
                <button className="btn-delete" onClick={() => deleteDomain(item.id)}>
                  Delete
                </button>
              </li>
            ))}
          </ul>
        ) : (
          <EmptyState tab="domains" onAdd={() => setShowForm(true)} />
        ),
      databases:
        databases.length > 0 ? (
          <ul className="item-list">
            {databases.map((item) => (
              <li key={item.id} className="item">
                <div className="item-info">
                  <span className="item-status" style={{ backgroundColor: '#9c27b0' }} />
                  <span className="item-name">{item.name}</span>
                  <span className="item-badge" style={{ backgroundColor: '#9c27b0' }}>
                    {item.db_type}
                  </span>
                  <span className="item-detail">
                    {item.host || 'localhost'}
                    {item.port && `:${item.port}`}
                  </span>
                </div>
                <button className="btn-delete" onClick={() => deleteDatabase(item.id)}>
                  Delete
                </button>
              </li>
            ))}
          </ul>
        ) : (
          <EmptyState tab="databases" onAdd={() => setShowForm(true)} />
        ),
      scripts:
        scripts.length > 0 ? (
          <ul className="item-list">
            {scripts.map((item) => (
              <li key={item.id} className="item">
                <div className="item-info">
                  <span className="item-status" style={{ backgroundColor: '#ff9800' }} />
                  <span className="item-name">{item.name}</span>
                  <span className="item-badge" style={{ backgroundColor: '#ff9800' }}>
                    {item.script_type}
                  </span>
                  <span className="item-detail item-command">
                    {item.command.length > 40 ? item.command.slice(0, 40) + '...' : item.command}
                  </span>
                </div>
                <button className="btn-delete" onClick={() => deleteScript(item.id)}>
                  Delete
                </button>
              </li>
            ))}
          </ul>
        ) : (
          <EmptyState tab="scripts" onAdd={() => setShowForm(true)} />
        ),
    };

    return lists[activeTab];
  };

  const hasItems = () => {
    switch (activeTab) {
      case 'projects':
        return projects.length > 0;
      case 'servers':
        return servers.length > 0;
      case 'domains':
        return domains.length > 0;
      case 'databases':
        return databases.length > 0;
      case 'scripts':
        return scripts.length > 0;
      default:
        return false;
    }
  };

  const totalCount = projects.length + servers.length + domains.length + databases.length + scripts.length;

  // ─────────────────────────────────────────────────────────────────────────────
  // Render
  // ─────────────────────────────────────────────────────────────────────────────

  return (
    <div className="container">
      <header>
        <h1>pctrl</h1>
        <p>Mission Control for Self-Hosters & Indie Devs</p>
        <div className="header-stats">
          <span className="stat">{totalCount} resources</span>
        </div>
      </header>

      <nav className="tabs">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            className={`tab ${activeTab === tab.id ? 'active' : ''}`}
            style={{ '--tab-color': tab.color } as React.CSSProperties}
            onClick={() => setActiveTab(tab.id)}
          >
            {tab.label}
          </button>
        ))}
      </nav>

      <main className="content">
        {error && (
          <div className="error-banner">
            {error}
            <button onClick={() => setError(null)}>×</button>
          </div>
        )}

        <div className="panel">
          <div className="panel-header">
            <div>
              <h2>{tabs.find((t) => t.id === activeTab)?.label}</h2>
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

// ─────────────────────────────────────────────────────────────────────────────
// Empty State Component
// ─────────────────────────────────────────────────────────────────────────────

function EmptyState({ tab, onAdd }: { tab: string; onAdd: () => void }) {
  const messages: Record<string, string> = {
    projects: 'No projects configured yet.',
    servers: 'No servers configured yet.',
    domains: 'No domains configured yet.',
    databases: 'No databases configured yet.',
    scripts: 'No scripts configured yet.',
  };

  return (
    <div className="empty-state">
      <p>{messages[tab]}</p>
      <button className="btn-primary" onClick={onAdd}>
        Add {tab.slice(0, -1)}
      </button>
    </div>
  );
}

export default App;
