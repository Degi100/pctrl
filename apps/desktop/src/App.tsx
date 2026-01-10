import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { open } from '@tauri-apps/api/dialog';
import { homeDir } from '@tauri-apps/api/path';
import { writeText } from '@tauri-apps/api/clipboard';

interface GeneratedKey {
  private_key_path: string;
  public_key_path: string;
  public_key_content: string;
}

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
  credential_id: string | null;
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

interface Credential {
  id: string;
  name: string;
  credential_type: string;
  data: {
    type: string;
    username?: string;
    port?: number;
    key_path?: string;
  };
}

interface ServerStatus {
  online: boolean;
  uptime: string | null;
  load: string | null;
  memory: string | null;
  disk: string | null;
  error: string | null;
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
  const [credentials, setCredentials] = useState<Credential[]>([]);

  // Form state
  const [showForm, setShowForm] = useState(false);
  const [formData, setFormData] = useState<Record<string, string>>({});

  // Server status state
  const [serverStatuses, setServerStatuses] = useState<Record<string, ServerStatus>>({});
  const [loadingStatus, setLoadingStatus] = useState<Record<string, boolean>>({});

  // Default SSH key path
  const [defaultKeyPath, setDefaultKeyPath] = useState<string>('');

  // Connection test state
  const [testHost, setTestHost] = useState<string>('');
  const [testResult, setTestResult] = useState<{ success: boolean; message: string } | null>(null);
  const [testing, setTesting] = useState(false);

  // Key generation state
  const [generatedKey, setGeneratedKey] = useState<GeneratedKey | null>(null);
  const [generating, setGenerating] = useState(false);
  const [copied, setCopied] = useState(false);

  // Load default key path on mount
  useEffect(() => {
    homeDir().then((home) => {
      setDefaultKeyPath(`${home}.ssh\\id_ed25519`);
    }).catch(() => {
      setDefaultKeyPath('C:\\Users\\USERNAME\\.ssh\\id_ed25519');
    });
  }, []);

  const getDefaultKeyPath = () => defaultKeyPath;

  const browseForKeyFile = async () => {
    try {
      const selected = await open({
        multiple: false,
        directory: false,
        defaultPath: defaultKeyPath ? defaultKeyPath.replace('id_ed25519', '') : undefined,
        title: 'SSH Key auswählen',
      });
      if (selected && typeof selected === 'string') {
        setFormData({ ...formData, key_path: selected });
      }
    } catch (err) {
      console.error('File dialog error:', err);
    }
  };

  const generateNewKey = async () => {
    if (!formData.name?.trim()) {
      setError('Bitte erst einen Namen eingeben');
      return;
    }
    setGenerating(true);
    setGeneratedKey(null);
    try {
      const result = await invoke<GeneratedKey>('generate_ssh_key', {
        name: formData.name.trim(),
      });
      setGeneratedKey(result);
      setFormData({ ...formData, key_path: result.private_key_path });
    } catch (err) {
      setError(`Key generation failed: ${err}`);
    } finally {
      setGenerating(false);
    }
  };

  const copyPublicKey = async () => {
    if (generatedKey) {
      await writeText(generatedKey.public_key_content);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  const testCredentialConnection = async (credentialId: string, host: string) => {
    if (!host.trim()) {
      setTestResult({ success: false, message: 'Bitte Host/IP eingeben' });
      return;
    }
    setTesting(true);
    setTestResult(null);
    try {
      const result = await invoke<string>('test_credential_connection', {
        credentialId,
        host: host.trim(),
      });
      setTestResult({ success: true, message: result });
    } catch (err) {
      setTestResult({ success: false, message: String(err) });
    } finally {
      setTesting(false);
    }
  };

  const tabs: Tab[] = [
    { id: 'projects', label: 'Projects', color: '#00bcd4' },
    { id: 'servers', label: 'Servers', color: '#4caf50' },
    { id: 'credentials', label: 'Credentials', color: '#e91e63' },
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

      const [projectsData, serversData, domainsData, databasesData, scriptsData, credentialsData] =
        await Promise.all([
          invoke<Project[]>('list_projects'),
          invoke<Server[]>('list_servers'),
          invoke<Domain[]>('list_domains'),
          invoke<DatabaseCredentials[]>('list_databases'),
          invoke<Script[]>('list_scripts'),
          invoke<Credential[]>('list_credentials'),
        ]);

      setProjects(projectsData);
      setServers(serversData);
      setDomains(domainsData);
      setDatabases(databasesData);
      setScripts(scriptsData);
      setCredentials(credentialsData);
    } catch (err) {
      setError(`Failed to load data: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  // ─────────────────────────────────────────────────────────────────────────────
  // Server Status
  // ─────────────────────────────────────────────────────────────────────────────

  const fetchServerStatus = async (serverId: string) => {
    setLoadingStatus((prev) => ({ ...prev, [serverId]: true }));
    try {
      const status = await invoke<ServerStatus>('get_server_status', { serverId });
      setServerStatuses((prev) => ({ ...prev, [serverId]: status }));
    } catch (err) {
      setServerStatuses((prev) => ({
        ...prev,
        [serverId]: { online: false, uptime: null, load: null, memory: null, disk: null, error: String(err) },
      }));
    } finally {
      setLoadingStatus((prev) => ({ ...prev, [serverId]: false }));
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
          credential_id: formData.credential_id || null,
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

  const addCredential = async () => {
    try {
      await invoke('add_credential', {
        data: {
          name: formData.name,
          credential_type: formData.credential_type || 'ssh_key',
          username: formData.username || 'root',
          port: formData.port ? parseInt(formData.port) : 22,
          key_path: formData.key_path || defaultKeyPath || null,
        },
      });
      closeForm();
      loadAllData();
    } catch (err) {
      setError(`Failed to add credential: ${err}`);
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

  const deleteCredential = async (id: string) => {
    try {
      await invoke('delete_credential', { id });
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
    setGeneratedKey(null);
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
      case 'credentials':
        addCredential();
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

  const getCredentialTypeLabel = (type: string) => {
    switch (type) {
      case 'SshKey':
        return 'SSH Key';
      case 'SshAgent':
        return 'SSH Agent';
      case 'ApiToken':
        return 'API Token';
      case 'BasicAuth':
        return 'Basic Auth';
      case 'OAuth':
        return 'OAuth';
      default:
        return type;
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
            placeholder="Name * (z.B. MeinServer)"
            title="Ein Name zur Identifikation deines Servers"
            value={formData.name || ''}
            onChange={(e) => setFormData({ ...formData, name: e.target.value })}
          />
          <input
            type="text"
            placeholder="Host * (IP oder Domain, z.B. 192.168.1.100)"
            title="IP-Adresse oder Domain des Servers"
            value={formData.host || ''}
            onChange={(e) => setFormData({ ...formData, host: e.target.value })}
          />
          <select
            value={formData.server_type || 'vps'}
            onChange={(e) => setFormData({ ...formData, server_type: e.target.value })}
            title="Art des Servers"
          >
            <option value="vps">VPS</option>
            <option value="dedicated">Dedicated</option>
            <option value="local">Local</option>
            <option value="cloud">Cloud</option>
          </select>
          <input
            type="text"
            placeholder="Provider (z.B. Hetzner, DigitalOcean)"
            title="Hosting-Anbieter (optional)"
            value={formData.provider || ''}
            onChange={(e) => setFormData({ ...formData, provider: e.target.value })}
          />
          <label className="form-label">
            SSH Credential (erst unter Credentials anlegen!)
          </label>
          <select
            value={formData.credential_id || ''}
            onChange={(e) => setFormData({ ...formData, credential_id: e.target.value })}
            title="SSH-Zugangsdaten zum Verbinden"
          >
            <option value="">-- Kein Credential (Status nicht verfügbar) --</option>
            {credentials
              .filter((c) => c.credential_type === 'SshKey' || c.credential_type === 'SshAgent')
              .map((c) => (
                <option key={c.id} value={c.id}>
                  {c.name} ({getCredentialTypeLabel(c.credential_type)})
                </option>
              ))}
          </select>
          {credentials.filter((c) => c.credential_type === 'SshKey' || c.credential_type === 'SshAgent').length === 0 && (
            <p className="form-hint">Tipp: Lege erst ein Credential im Credentials-Tab an!</p>
          )}
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
      credentials: (
        <>
          <input
            type="text"
            placeholder="Name * (z.B. Mein SSH)"
            title="Ein Name für diese Zugangsdaten"
            value={formData.name || ''}
            onChange={(e) => setFormData({ ...formData, name: e.target.value })}
          />
          <select
            value={formData.credential_type || 'ssh_key'}
            onChange={(e) => setFormData({ ...formData, credential_type: e.target.value })}
            title="SSH Key liest die Datei direkt, SSH Agent nutzt den Windows SSH-Agent"
          >
            <option value="ssh_key">SSH Key (Key-Datei angeben)</option>
            <option value="ssh_agent">SSH Agent (Keys im RAM, nach Neustart weg)</option>
          </select>
          <input
            type="text"
            placeholder="Username"
            title="SSH-Benutzername für den Server (meist root)"
            value={formData.username ?? 'root'}
            onChange={(e) => setFormData({ ...formData, username: e.target.value })}
          />
          <input
            type="number"
            placeholder="Port (Standard: 22)"
            title="SSH-Port, normalerweise 22"
            value={formData.port || ''}
            onChange={(e) => setFormData({ ...formData, port: e.target.value })}
          />
          {(formData.credential_type || 'ssh_key') === 'ssh_key' && (
            <>
              <div className="key-options">
                <button
                  type="button"
                  className="btn-generate"
                  onClick={generateNewKey}
                  disabled={generating || !formData.name?.trim()}
                  title="Neuen RSA Key generieren (funktioniert garantiert)"
                >
                  {generating ? 'Generiere...' : 'Neuen Key generieren'}
                </button>
                <span className="or-divider">oder</span>
              </div>
              <div className="input-with-button">
                <input
                  type="text"
                  placeholder="Bestehenden Key Path angeben"
                  title="Pfad zur privaten SSH-Key-Datei"
                  value={formData.key_path || getDefaultKeyPath()}
                  onChange={(e) => setFormData({ ...formData, key_path: e.target.value })}
                />
                <button
                  type="button"
                  className="btn-browse"
                  onClick={() => browseForKeyFile()}
                  title="Datei auswählen"
                >
                  ...
                </button>
              </div>
              {generatedKey && (
                <div className="generated-key-info">
                  <p className="success-text">Key erstellt: {generatedKey.private_key_path}</p>
                  <p className="key-instruction">Public Key auf Server kopieren:</p>
                  <div className="public-key-box">
                    <code>{generatedKey.public_key_content.slice(0, 50)}...</code>
                    <button
                      type="button"
                      className="btn-copy"
                      onClick={copyPublicKey}
                    >
                      {copied ? 'Kopiert!' : 'Kopieren'}
                    </button>
                  </div>
                  <p className="form-hint">
                    Füge diesen Key auf dem Server ein: ~/.ssh/authorized_keys
                  </p>
                </div>
              )}
              {!generatedKey && (
                <p className="form-hint">
                  Hinweis: ED25519 Keys funktionieren evtl. nicht - RSA empfohlen!
                </p>
              )}
            </>
          )}
          {formData.credential_type === 'ssh_agent' && (
            <p className="form-hint">
              SSH Agent speichert Keys nur im RAM - nach PC-Neustart musst du ssh-add erneut ausführen!
            </p>
          )}
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
            {servers.map((item) => {
              const status = serverStatuses[item.id];
              const isLoading = loadingStatus[item.id];
              const cred = credentials.find((c) => c.id === item.credential_id);

              return (
                <li key={item.id} className="item server-item">
                  <div className="item-info">
                    <span
                      className="item-status"
                      style={{
                        backgroundColor: status?.online ? '#4caf50' : status?.error ? '#f44336' : '#999',
                      }}
                    />
                    <span className="item-name">{item.name}</span>
                    <span className="item-detail">{item.host}</span>
                    <span className="item-badge">{item.server_type}</span>
                    {cred && <span className="item-cred">[{cred.name}]</span>}
                  </div>
                  <div className="item-actions">
                    {item.credential_id && (
                      <button
                        className="btn-status"
                        onClick={() => fetchServerStatus(item.id)}
                        disabled={isLoading}
                      >
                        {isLoading ? '...' : 'Status'}
                      </button>
                    )}
                    <button className="btn-delete" onClick={() => deleteServer(item.id)}>
                      Delete
                    </button>
                  </div>
                  {status && (
                    <div className="server-status">
                      {status.online ? (
                        <>
                          <span className="status-online">Online</span>
                          {status.uptime && <span>Uptime: {status.uptime}</span>}
                          {status.load && <span>Load: {status.load}</span>}
                          {status.memory && <span>Memory: {status.memory}</span>}
                          {status.disk && <span>Disk: {status.disk}</span>}
                        </>
                      ) : (
                        <span className="status-offline">Offline: {status.error}</span>
                      )}
                    </div>
                  )}
                </li>
              );
            })}
          </ul>
        ) : (
          <EmptyState tab="servers" onAdd={() => setShowForm(true)} />
        ),
      credentials:
        credentials.length > 0 ? (
          <ul className="item-list">
            {credentials.map((item) => (
              <li key={item.id} className="item credential-item">
                <div className="item-row">
                  <div className="item-info">
                    <span className="item-status" style={{ backgroundColor: '#e91e63' }} />
                    <span className="item-name">{item.name}</span>
                    <span className="item-badge" style={{ backgroundColor: '#e91e63' }}>
                      {getCredentialTypeLabel(item.credential_type)}
                    </span>
                    <span className="item-detail">
                      {item.data.username}@:{item.data.port || 22}
                    </span>
                  </div>
                  <button className="btn-delete" onClick={() => deleteCredential(item.id)}>
                    Delete
                  </button>
                </div>
                <div className="test-connection">
                  <input
                    type="text"
                    placeholder="Server-IP eingeben (z.B. 192.168.1.100)"
                    title="Die IP-Adresse eines Servers zum Testen der Verbindung"
                    className="test-host-input"
                    value={testHost}
                    onChange={(e) => setTestHost(e.target.value)}
                  />
                  <button
                    className="btn-test"
                    onClick={() => testCredentialConnection(item.id, testHost)}
                    disabled={testing}
                  >
                    {testing ? '...' : 'Test'}
                  </button>
                </div>
                {testResult && (
                  <div className={`test-result ${testResult.success ? 'success' : 'error'}`}>
                    {testResult.message}
                  </div>
                )}
              </li>
            ))}
          </ul>
        ) : (
          <EmptyState tab="credentials" onAdd={() => setShowForm(true)} />
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
      case 'credentials':
        return credentials.length > 0;
      default:
        return false;
    }
  };

  const totalCount =
    projects.length + servers.length + domains.length + databases.length + scripts.length + credentials.length;

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
            <button onClick={() => setError(null)}>x</button>
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
    credentials: 'No credentials configured yet. Add one to connect to servers.',
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
