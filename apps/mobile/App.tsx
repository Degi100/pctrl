import { StatusBar } from 'expo-status-bar';
import { StyleSheet, Text, View, ScrollView, TouchableOpacity } from 'react-native';

export default function App() {
  return (
    <View style={styles.container}>
      <StatusBar style="light" />
      
      <View style={styles.header}>
        <Text style={styles.title}>pctrl</Text>
        <Text style={styles.subtitle}>Mission Control</Text>
      </View>

      <ScrollView style={styles.content}>
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>🔐 SSH Connections</Text>
          <View style={styles.card}>
            <Text style={styles.cardText}>No connections configured</Text>
            <TouchableOpacity style={styles.button}>
              <Text style={styles.buttonText}>Add Connection</Text>
            </TouchableOpacity>
          </View>
        </View>

        <View style={styles.section}>
          <Text style={styles.sectionTitle}>🐳 Docker Containers</Text>
          <View style={styles.card}>
            <Text style={styles.cardText}>No hosts configured</Text>
            <TouchableOpacity style={styles.button}>
              <Text style={styles.buttonText}>Add Host</Text>
            </TouchableOpacity>
          </View>
        </View>

        <View style={styles.section}>
          <Text style={styles.sectionTitle}>🚀 Coolify Deployments</Text>
          <View style={styles.card}>
            <Text style={styles.cardText}>No instances configured</Text>
            <TouchableOpacity style={styles.button}>
              <Text style={styles.buttonText}>Add Instance</Text>
            </TouchableOpacity>
          </View>
        </View>

        <View style={styles.section}>
          <Text style={styles.sectionTitle}>📦 Git Releases</Text>
          <View style={styles.card}>
            <Text style={styles.cardText}>No repositories configured</Text>
            <TouchableOpacity style={styles.button}>
              <Text style={styles.buttonText}>Add Repository</Text>
            </TouchableOpacity>
          </View>
        </View>
      </ScrollView>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#1a1a1a',
  },
  header: {
    backgroundColor: '#667eea',
    paddingTop: 60,
    paddingBottom: 30,
    paddingHorizontal: 20,
    alignItems: 'center',
  },
  title: {
    fontSize: 36,
    fontWeight: 'bold',
    color: '#fff',
    marginBottom: 5,
  },
  subtitle: {
    fontSize: 18,
    color: '#fff',
    opacity: 0.9,
  },
  content: {
    flex: 1,
    padding: 20,
  },
  section: {
    marginBottom: 30,
  },
  sectionTitle: {
    fontSize: 20,
    fontWeight: '600',
    color: '#667eea',
    marginBottom: 15,
  },
  card: {
    backgroundColor: '#2a2a2a',
    borderRadius: 12,
    padding: 20,
    alignItems: 'center',
  },
  cardText: {
    color: '#999',
    fontSize: 16,
    marginBottom: 15,
  },
  button: {
    backgroundColor: '#667eea',
    paddingVertical: 12,
    paddingHorizontal: 24,
    borderRadius: 8,
  },
  buttonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
});
