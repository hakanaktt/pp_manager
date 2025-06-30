<template>
  <div class="app">
    <header class="header">
      <h1>🎯 Process Priority Manager</h1>
    </header>

    <main class="main">
      <!-- System Information -->
      <section class="system-info">
        <div class="info-item">
          <span class="info-label">🖥️ System:</span>
          <span class="info-value">{{ systemInfo.cpu_count }} CPU cores (Mask: {{ systemInfo.affinity_mask }})</span>
        </div>
        <div class="info-item">
          <span class="info-label">🛡️ Admin Status:</span>
          <span :class="['info-value', systemInfo.is_admin ? 'admin-yes' : 'admin-no']">
            {{ systemInfo.is_admin ? 'Running as Administrator' : 'Not running as Administrator' }}
          </span>
          <span v-if="!systemInfo.is_admin" class="admin-warning">
            (May have limited access to system processes)
          </span>
        </div>
      </section>

      <!-- Tracked Processes Section -->
      <section v-if="trackedProcesses.length > 0" class="tracked-processes">
        <h2>🎯 Tracked Process Instances</h2>
        <div class="process-list">
          <div
            v-for="process in trackedProcesses"
            :key="process.pid"
            class="process-item"
          >
            <div class="process-info">
              <span class="process-pid">PID: {{ process.pid }}</span>
              <span class="process-status">🟢 Active</span>
            </div>
            <div class="process-details">
              <span v-if="process.last_applied_affinity" class="process-detail">
                Affinity: {{ formatHex(process.last_applied_affinity) }}
              </span>
              <span v-if="process.last_applied_priority" class="process-detail">
                Priority: {{ formatHex(process.last_applied_priority) }}
              </span>
            </div>
            <div class="process-actions">
              <button
                class="btn-process-action"
                @click="showProcessDetails(process)"
                title="Show process details"
              >
                📊 Details
              </button>
              <button
                class="btn-process-action btn-exclude"
                @click="excludeProcess(process.pid)"
                title="Exclude this process from monitoring"
                :disabled="isMonitoring"
              >
                ❌ Exclude
              </button>
            </div>
          </div>
        </div>
        <div class="process-summary">
          <span>Total instances: {{ trackedProcesses.length }}</span>
          <span v-if="excludedProcesses.size > 0" class="excluded-count">
            ({{ excludedProcesses.size }} excluded)
          </span>
          <button
            v-if="excludedProcesses.size > 0"
            @click="clearExcludedProcesses"
            class="btn-clear-excluded"
          >
            🔄 Show All
          </button>
        </div>
      </section>



      <!-- Multi-Process Management Section -->
      <section class="processes-section">
        <div class="section-header">
          <div class="header-content">
            <h2>🎯 Process Management</h2>
            <p class="section-subtitle">Configure multiple processes with individual CPU and priority settings</p>
          </div>
          <button
            @click="showAddProcess = true"
            :disabled="isMonitoring"
            class="add-process-btn"
          >
            <span class="btn-icon">+</span>
            Add Process
          </button>
        </div>

        <!-- Add Process Modal/Form -->
        <div v-if="showAddProcess" class="add-process-modal">
          <div class="modal-content">
            <div class="modal-header">
              <h3>Add New Process</h3>
              <button @click="cancelAddProcess" class="close-btn">×</button>
            </div>
            <div class="modal-body">
              <div class="input-group">
                <label for="process-name">Process Name</label>
                <input
                  id="process-name"
                  v-model="newProcessName"
                  type="text"
                  placeholder="e.g., notepad.exe, chrome.exe, game.exe"
                  class="modern-input"
                  @keyup.enter="addNewProcess"
                />
                <small class="input-hint">Enter the executable name (e.g., "notepad.exe")</small>
              </div>
            </div>
            <div class="modal-footer">
              <button @click="cancelAddProcess" class="btn-secondary">Cancel</button>
              <button @click="addNewProcess" class="btn-primary">Add Process</button>
            </div>
          </div>
        </div>

        <!-- Process List -->
        <div class="processes-container">
          <div v-if="processConfigs.length === 0" class="empty-state">
            <div class="empty-icon">📋</div>
            <h3>No Processes Configured</h3>
            <p>Add your first process to start monitoring CPU usage and priority settings</p>
            <button
              @click="showAddProcess = true"
              :disabled="isMonitoring"
              class="btn-primary"
            >
              Add Your First Process
            </button>
          </div>

          <div v-else class="process-list">
            <div
              v-for="(processConfig, index) in processConfigs"
              :key="processConfig.name"
              :class="['process-card', { 'disabled': !processConfig.enabled }]"
            >
              <!-- Process Header -->
              <div class="process-header">
                <div class="process-info">
                  <div class="process-name">
                    <span class="process-icon">⚙️</span>
                    <span class="name">{{ processConfig.name }}</span>
                  </div>
                  <div class="process-meta">
                    <span :class="['status-badge', processConfig.enabled ? 'active' : 'inactive']">
                      {{ processConfig.enabled ? 'Active' : 'Inactive' }}
                    </span>
                    <span class="core-count">
                      {{ processConfig.core_selections.filter(Boolean).length }} cores
                    </span>
                    <span class="priority-display">
                      {{ getPriorityName(processConfig.priority_class) }}
                    </span>
                  </div>
                </div>
                <div class="process-actions">
                  <button
                    @click="toggleProcessEnabled(index)"
                    :disabled="isMonitoring"
                    :class="['action-btn', 'toggle-btn', processConfig.enabled ? 'disable' : 'enable']"
                    :title="processConfig.enabled ? 'Disable process' : 'Enable process'"
                  >
                    {{ processConfig.enabled ? '⏸️' : '▶️' }}
                  </button>
                  <button
                    @click="editProcess(index)"
                    :disabled="isMonitoring"
                    class="action-btn edit-btn"
                    title="Edit settings"
                  >
                    ✏️
                  </button>
                  <button
                    @click="removeProcess(index)"
                    :disabled="isMonitoring"
                    class="action-btn remove-btn"
                    title="Remove process"
                  >
                    🗑️
                  </button>
                </div>
              </div>

              <!-- Expandable Settings (when editing) -->
              <div v-if="editingIndex === index" class="process-settings">
                <div class="settings-grid">
                  <!-- CPU Core Selection -->
                  <div class="setting-group">
                    <label class="setting-label">CPU Cores</label>
                    <div class="cores-container">
                      <div class="cores-grid">
                        <label
                          v-for="(selected, coreIndex) in processConfig.core_selections"
                          :key="coreIndex"
                          :class="['core-chip', { 'selected': selected }]"
                        >
                          <input
                            type="checkbox"
                            v-model="processConfig.core_selections[coreIndex]"
                            :disabled="isMonitoring"
                          />
                          <span class="core-number">{{ coreIndex }}</span>
                        </label>
                      </div>
                      <small class="setting-hint">
                        Selected: {{ processConfig.core_selections.filter(Boolean).length }} of {{ systemInfo.cpu_count }} cores
                      </small>
                    </div>
                  </div>

                  <!-- Priority Selection -->
                  <div class="setting-group">
                    <label class="setting-label">Priority Class</label>
                    <select
                      v-model="processConfig.priority_class"
                      :disabled="isMonitoring"
                      class="modern-select"
                    >
                      <option
                        v-for="option in priorityOptions"
                        :key="option.value"
                        :value="option.value"
                      >
                        {{ option.name }}
                      </option>
                    </select>
                    <small class="setting-hint">Higher priority gets more CPU time</small>
                  </div>
                </div>

                <div class="settings-actions">
                  <button @click="saveProcessSettings(index)" class="btn-primary">Save Changes</button>
                  <button @click="cancelEdit()" class="btn-secondary">Cancel</button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div v-if="isMonitoring && processConfigs.length > 0" class="monitoring-notice">
          <span class="notice-icon">ℹ️</span>
          Stop monitoring to modify process configurations
        </div>
      </section>

      <!-- Control Buttons -->
      <section class="controls">
        <div class="control-buttons">
          <button 
            v-if="isMonitoring"
            @click="stopMonitoring"
            class="btn btn-stop"
          >
            ⏹ Stop Monitoring
          </button>
          <button 
            v-else
            :disabled="!canStartMonitoring"
            @click="startMonitoring"
            class="btn btn-start"
          >
            ▶ Start Monitoring
          </button>
          
          <span :class="['status', isMonitoring ? 'status-active' : 'status-inactive']">
            {{ isMonitoring ? '🟢 Monitoring Active' : '🔴 Monitoring Stopped' }}
          </span>
        </div>

        <div class="secondary-controls">
          <button @click="clearLogs" class="btn btn-secondary">🗑 Clear Logs</button>
          <button @click="minimizeToTray" class="btn btn-secondary">📱 Minimize to Tray</button>
          <label class="checkbox-label">
            <input v-model="autoScroll" type="checkbox" />
            Auto-scroll logs
          </label>
          <span class="log-count">📊 {{ logs.length }} log entries</span>
        </div>
      </section>

      <!-- Tips -->
      <section class="tips">
        <div class="tip">
          💡 Tip: Use Ctrl+Shift+P to show/hide this window, or use the system tray icon.
        </div>
      </section>

      <!-- Logs Section -->
      <section class="logs-section">
        <h2>📋 Activity Logs</h2>
        <div class="logs-container" ref="logsContainer">
          <div v-if="logs.length === 0" class="no-logs">
            No logs yet. Start monitoring to see activity.
          </div>
          <div v-else class="logs">
            <div 
              v-for="(log, index) in displayedLogs" 
              :key="index"
              :class="['log-entry', `log-${log.level.toLowerCase()}`]"
            >
              <span class="log-level">[{{ log.level }}]</span>
              <span class="log-time">{{ formatTime(log.timestamp) }}</span>
              <span class="log-message">{{ log.message }}</span>
            </div>
          </div>
        </div>
      </section>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface SystemInfo {
  cpu_count: number
  affinity_mask: string
  is_admin: boolean
}

interface ProcessConfig {
  name: string
  core_selections: boolean[]
  priority_class: number
  enabled: boolean
}

interface Config {
  target_process: string
  core_selections: boolean[]
  priority_class: number
  processes: ProcessConfig[]
}

interface LogEntry {
  timestamp: string
  message: string
  level: string
}

interface PriorityOption {
  name: string
  value: number
}



interface TrackedProcess {
  pid: number
  last_applied_affinity: number | null
  last_applied_priority: number | null
}

// Reactive state
const systemInfo = ref<SystemInfo>({
  cpu_count: 0,
  affinity_mask: '0x0',
  is_admin: false
})

const config = ref<Config>({
  target_process: '',
  core_selections: [],
  priority_class: 0,
  processes: []
})

const processConfigs = ref<ProcessConfig[]>([])
const newProcessName = ref('')
const showAddProcess = ref(false)
const editingIndex = ref<number | null>(null)

const isMonitoring = ref(false)
const logs = ref<LogEntry[]>([])
const autoScroll = ref(true)
const logsContainer = ref<HTMLElement>()
const trackedProcesses = ref<TrackedProcess[]>([])
const excludedProcesses = ref<Set<number>>(new Set())

// Constants
const priorityOptions: PriorityOption[] = [
  { name: 'IDLE', value: 0x00000040 },
  { name: 'BELOW_NORMAL', value: 0x00004000 },
  { name: 'NORMAL', value: 0x00000020 },
  { name: 'ABOVE_NORMAL', value: 0x00008000 },
  { name: 'HIGH', value: 0x00000080 },
  { name: 'REALTIME', value: 0x00000100 }
]



// Computed properties
const canStartMonitoring = computed(() => {
  if (isMonitoring.value) return false

  // Check if we have any enabled processes
  return processConfigs.value.some(p => p.enabled)
})

const displayedLogs = computed(() => {
  // Show last 20 logs to prevent UI slowdown
  return logs.value.slice(-20)
})

// Methods




const formatHex = (value: number): string => {
  return `0x${value.toString(16).toUpperCase().padStart(8, '0')}`
}

const formatTime = (timestamp: string): string => {
  return new Date(timestamp).toLocaleTimeString()
}

const loadSystemInfo = async () => {
  try {
    systemInfo.value = await invoke('get_system_info')
  } catch (error) {
    console.error('Failed to load system info:', error)
  }
}

const loadConfig = async () => {
  try {
    config.value = await invoke('get_config')
  } catch (error) {
    console.error('Failed to load config:', error)
  }
}



const startMonitoring = async () => {
  try {
    await invoke('start_monitoring')
    isMonitoring.value = true

    startLogPolling()
  } catch (error) {
    console.error('Failed to start monitoring:', error)
    alert(`Failed to start monitoring: ${error}`)
  }
}

const stopMonitoring = async () => {
  try {
    await invoke('stop_monitoring')
    isMonitoring.value = false
  } catch (error) {
    console.error('Failed to stop monitoring:', error)
  }
}

const clearLogs = async () => {
  try {
    await invoke('clear_logs')
    logs.value = []
  } catch (error) {
    console.error('Failed to clear logs:', error)
  }
}

const minimizeToTray = async () => {
  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window')
    const appWindow = getCurrentWindow()
    await appWindow.hide()
  } catch (error) {
    console.error('Failed to minimize to tray:', error)
  }
}



const checkMonitoringStatus = async () => {
  try {
    isMonitoring.value = await invoke('get_monitoring_status')
  } catch (error) {
    console.error('Failed to check monitoring status:', error)
  }
}

const loadLogs = async () => {
  try {
    logs.value = await invoke('get_logs')
    if (autoScroll.value) {
      await nextTick()
      scrollToBottom()
    }
  } catch (error) {
    console.error('Failed to load logs:', error)
  }
}

const loadTrackedProcesses = async () => {
  try {
    const allProcesses = await invoke('get_tracked_processes') as TrackedProcess[]
    // Filter out excluded processes
    trackedProcesses.value = allProcesses.filter(p => !excludedProcesses.value.has(p.pid))
  } catch (error) {
    console.error('Failed to load tracked processes:', error)
  }
}

const showProcessDetails = (process: TrackedProcess) => {
  const details = [
    `Process ID: ${process.pid}`,
    `Last Applied Affinity: ${process.last_applied_affinity ? formatHex(process.last_applied_affinity) : 'None'}`,
    `Last Applied Priority: ${process.last_applied_priority ? formatHex(process.last_applied_priority) : 'None'}`
  ].join('\n')

  alert(`Process Details:\n\n${details}`)
}

const excludeProcess = (pid: number) => {
  if (confirm(`Are you sure you want to exclude process PID ${pid} from monitoring?\n\nThis will stop managing this specific process instance.`)) {
    excludedProcesses.value.add(pid)
    // Remove from current tracked processes
    trackedProcesses.value = trackedProcesses.value.filter(p => p.pid !== pid)
  }
}

const clearExcludedProcesses = () => {
  excludedProcesses.value.clear()
  loadTrackedProcesses()
}

// Multi-process management functions
const loadProcessConfigs = async () => {
  try {
    console.log('Loading process configs...')
    const configs = await invoke('get_process_configs') as ProcessConfig[]
    console.log('Loaded process configs:', configs)
    processConfigs.value = configs
  } catch (error) {
    console.error('Failed to load process configs:', error)
    alert(`Failed to load process configs: ${error}`)
  }
}

const addNewProcess = async () => {
  if (!newProcessName.value.trim()) {
    alert('Please enter a process name')
    return
  }

  try {
    // Create default core selections (same as system default)
    const defaultCoreSelections = Array.from({ length: systemInfo.value.cpu_count }, () => false)
    if (systemInfo.value.cpu_count > 4) defaultCoreSelections[4] = true
    if (systemInfo.value.cpu_count > 5) defaultCoreSelections[5] = true

    console.log('Adding process:', {
      name: newProcessName.value.trim(),
      coreSelections: defaultCoreSelections,
      priorityClass: 0x00008000
    })

    await invoke('add_process_config', {
      name: newProcessName.value.trim(),
      coreSelections: defaultCoreSelections,
      priorityClass: 0x00008000 // ABOVE_NORMAL_PRIORITY_CLASS
    })

    await loadProcessConfigs()
    newProcessName.value = ''
    showAddProcess.value = false
  } catch (error) {
    console.error('Failed to add process:', error)
    alert(`Failed to add process: ${error}`)
  }
}

const cancelAddProcess = () => {
  newProcessName.value = ''
  showAddProcess.value = false
}

const removeProcess = async (index: number) => {
  const processConfig = processConfigs.value[index]
  if (confirm(`Are you sure you want to remove "${processConfig.name}" from monitoring?`)) {
    try {
      await invoke('remove_process_config', { name: processConfig.name })
      await loadProcessConfigs()
    } catch (error) {
      alert(`Failed to remove process: ${error}`)
    }
  }
}

const toggleProcessEnabled = async (index: number) => {
  const processConfig = processConfigs.value[index]
  try {
    await invoke('update_process_config', {
      name: processConfig.name,
      coreSelections: processConfig.core_selections,
      priorityClass: processConfig.priority_class,
      enabled: !processConfig.enabled
    })
    await loadProcessConfigs()
  } catch (error) {
    alert(`Failed to update process: ${error}`)
  }
}

const editProcess = (index: number) => {
  editingIndex.value = index
}

const cancelEdit = () => {
  editingIndex.value = null
  // Reload to reset any unsaved changes
  loadProcessConfigs()
}

const saveProcessSettings = async (index: number) => {
  const processConfig = processConfigs.value[index]

  // Validate that at least one core is selected
  if (!processConfig.core_selections.some(Boolean)) {
    alert('Please select at least one CPU core')
    return
  }

  try {
    await invoke('update_process_config', {
      name: processConfig.name,
      coreSelections: processConfig.core_selections,
      priorityClass: processConfig.priority_class,
      enabled: processConfig.enabled
    })
    editingIndex.value = null
    await loadProcessConfigs()
  } catch (error) {
    alert(`Failed to save settings: ${error}`)
  }
}

const getPriorityName = (priorityClass: number): string => {
  const option = priorityOptions.find(opt => opt.value === priorityClass)
  return option ? option.name : 'Unknown'
}

const scrollToBottom = () => {
  if (logsContainer.value) {
    logsContainer.value.scrollTop = logsContainer.value.scrollHeight
  }
}

let logPollingInterval: number | null = null

const startLogPolling = () => {
  if (logPollingInterval) return
  
  logPollingInterval = setInterval(async () => {
    await loadLogs()
    await loadTrackedProcesses()
    await loadProcessConfigs()
    await checkMonitoringStatus()
  }, 1000)
}



// Watchers
watch(autoScroll, async (newValue) => {
  if (newValue) {
    await nextTick()
    scrollToBottom()
  }
})

// Lifecycle
onMounted(async () => {
  await loadSystemInfo()
  await loadConfig()
  await loadProcessConfigs()
  await checkMonitoringStatus()
  await loadLogs()
  await loadTrackedProcesses()

  if (isMonitoring.value) {
    startLogPolling()
  }
})
</script>

<style scoped>
.app {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  max-width: 100%;
  margin: 0;
  padding: 16px;
  background: #f5f5f5;
  min-height: 100vh;
  box-sizing: border-box;
}

.header {
  text-align: center;
  margin-bottom: 20px;
}

.header h1 {
  margin: 0;
  color: #333;
  font-size: 24px;
}

.main {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

/* System Info */
.system-info {
  background: white;
  padding: 16px;
  border-radius: 8px;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

/* Tracked Processes */
.tracked-processes {
  background: white;
  padding: 16px;
  border-radius: 8px;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

.tracked-processes h2 {
  margin: 0 0 16px 0;
  color: #333;
  font-size: 18px;
}

.process-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-bottom: 12px;
}

.process-item {
  background: #f9fafb;
  padding: 12px;
  border-radius: 6px;
  border-left: 4px solid #22c55e;
}

.process-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.process-pid {
  font-weight: 600;
  color: #333;
  font-family: 'Courier New', monospace;
}

.process-status {
  font-size: 14px;
  color: #22c55e;
}

.process-details {
  display: flex;
  gap: 16px;
  font-size: 14px;
  color: #6b7280;
}

.process-detail {
  font-family: 'Courier New', monospace;
}

.process-summary {
  font-size: 14px;
  color: #6b7280;
  text-align: center;
  padding: 8px;
  background: #f3f4f6;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
}

.excluded-count {
  color: #f59e0b;
  font-weight: 600;
}

.process-actions {
  display: flex;
  gap: 8px;
  margin-top: 8px;
}

.btn-process-action {
  padding: 4px 8px;
  border: 1px solid #d1d5db;
  border-radius: 4px;
  background: white;
  cursor: pointer;
  font-size: 12px;
  transition: background-color 0.2s;
}

.btn-process-action:hover:not(:disabled) {
  background: #f3f4f6;
}

.btn-process-action:disabled {
  background: #f3f4f6;
  color: #6b7280;
  cursor: not-allowed;
}

.btn-exclude {
  color: #ef4444;
  border-color: #ef4444;
}

.btn-exclude:hover:not(:disabled) {
  background: #fef2f2;
}

.btn-clear-excluded {
  padding: 4px 8px;
  border: 1px solid #22c55e;
  border-radius: 4px;
  background: white;
  color: #22c55e;
  cursor: pointer;
  font-size: 12px;
  transition: background-color 0.2s;
}

.btn-clear-excluded:hover {
  background: #f0fdf4;
}

.info-item {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.info-item:last-child {
  margin-bottom: 0;
}

.info-label {
  font-weight: 600;
  color: #555;
}

.info-value {
  color: #333;
}

.admin-yes {
  color: #22c55e;
}

.admin-no {
  color: #f59e0b;
}

.admin-warning {
  color: #6b7280;
  font-size: 14px;
}



/* Modern Process Management Section */
.processes-section {
  background: white;
  border-radius: 12px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  overflow: hidden;
  margin-top: 16px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  padding: 24px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}

.header-content h2 {
  margin: 0 0 4px 0;
  font-size: 20px;
  font-weight: 600;
}

.section-subtitle {
  margin: 0;
  opacity: 0.9;
  font-size: 14px;
}

.add-process-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  background: rgba(255, 255, 255, 0.2);
  color: white;
  border: 1px solid rgba(255, 255, 255, 0.3);
  padding: 10px 16px;
  border-radius: 8px;
  cursor: pointer;
  font-weight: 500;
  transition: all 0.2s ease;
}

.add-process-btn:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.3);
  transform: translateY(-1px);
}

.add-process-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-icon {
  font-size: 16px;
  font-weight: bold;
}

/* Add Process Modal */
.add-process-modal {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-content {
  background: white;
  border-radius: 12px;
  width: 90%;
  max-width: 500px;
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1);
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px;
  border-bottom: 1px solid #e5e7eb;
}

.modal-header h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: #111827;
}

.close-btn {
  background: none;
  border: none;
  font-size: 24px;
  cursor: pointer;
  color: #6b7280;
  padding: 0;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
}

.close-btn:hover {
  background: #f3f4f6;
  color: #374151;
}

.modal-body {
  padding: 24px;
}

.input-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.input-group label {
  font-weight: 500;
  color: #374151;
  font-size: 14px;
}

.modern-input {
  padding: 12px 16px;
  border: 2px solid #e5e7eb;
  border-radius: 8px;
  font-size: 14px;
  transition: border-color 0.2s ease;
}

.modern-input:focus {
  outline: none;
  border-color: #3b82f6;
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

.input-hint {
  color: #6b7280;
  font-size: 12px;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding: 20px 24px;
  border-top: 1px solid #e5e7eb;
  background: #f9fafb;
}

.btn-primary {
  background: #3b82f6;
  color: white;
  padding: 10px 20px;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  font-weight: 500;
  transition: background-color 0.2s ease;
}

.btn-primary:hover {
  background: #2563eb;
}

.btn-secondary {
  background: #f3f4f6;
  color: #374151;
  padding: 10px 20px;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  font-weight: 500;
  transition: background-color 0.2s ease;
}

.btn-secondary:hover {
  background: #e5e7eb;
}

/* Process List Container */
.processes-container {
  padding: 24px;
}

.empty-state {
  text-align: center;
  padding: 48px 24px;
  color: #6b7280;
}

.empty-icon {
  font-size: 48px;
  margin-bottom: 16px;
}

.empty-state h3 {
  margin: 0 0 8px 0;
  color: #374151;
  font-size: 18px;
}

.empty-state p {
  margin: 0 0 24px 0;
  font-size: 14px;
}

/* Process List */
.process-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.process-card {
  border: 2px solid #e5e7eb;
  border-radius: 12px;
  transition: all 0.2s ease;
  overflow: hidden;
}

.process-card:hover {
  border-color: #d1d5db;
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
}

.process-card.disabled {
  opacity: 0.6;
  border-color: #f3f4f6;
}

.process-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px;
  background: #fafafa;
}

.process-info {
  flex: 1;
}

.process-name {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 8px;
}

.process-icon {
  font-size: 18px;
}

.name {
  font-weight: 600;
  font-size: 16px;
  color: #111827;
}

.process-meta {
  display: flex;
  gap: 12px;
  align-items: center;
  flex-wrap: wrap;
}

.status-badge {
  padding: 4px 12px;
  border-radius: 20px;
  font-size: 12px;
  font-weight: 500;
}

.status-badge.active {
  background: #dcfce7;
  color: #166534;
}

.status-badge.inactive {
  background: #fee2e2;
  color: #991b1b;
}

.core-count,
.priority-display {
  font-size: 12px;
  color: #6b7280;
  background: #f3f4f6;
  padding: 4px 8px;
  border-radius: 6px;
}

.process-actions {
  display: flex;
  gap: 8px;
}

.action-btn {
  width: 36px;
  height: 36px;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
  transition: all 0.2s ease;
}

.toggle-btn.enable {
  background: #dcfce7;
  color: #166534;
}

.toggle-btn.disable {
  background: #fef3c7;
  color: #d97706;
}

.edit-btn {
  background: #dbeafe;
  color: #1d4ed8;
}

.remove-btn {
  background: #fee2e2;
  color: #dc2626;
}

.action-btn:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Process Settings (Expanded) */
.process-settings {
  padding: 24px;
  background: white;
  border-top: 1px solid #e5e7eb;
}

.settings-grid {
  display: grid;
  gap: 24px;
  margin-bottom: 24px;
}

.setting-group {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.setting-label {
  font-weight: 600;
  color: #374151;
  font-size: 14px;
}

.cores-container {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.cores-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(40px, 1fr));
  gap: 8px;
  max-width: 400px;
}

.core-chip {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  border: 2px solid #e5e7eb;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
  background: white;
}

.core-chip input {
  position: absolute;
  opacity: 0;
  pointer-events: none;
}

.core-chip.selected {
  border-color: #3b82f6;
  background: #3b82f6;
  color: white;
}

.core-chip:hover {
  border-color: #3b82f6;
}

.core-number {
  font-weight: 500;
  font-size: 12px;
}

.modern-select {
  padding: 12px 16px;
  border: 2px solid #e5e7eb;
  border-radius: 8px;
  font-size: 14px;
  background: white;
  cursor: pointer;
  transition: border-color 0.2s ease;
}

.modern-select:focus {
  outline: none;
  border-color: #3b82f6;
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

.setting-hint {
  color: #6b7280;
  font-size: 12px;
}

.settings-actions {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
}

/* Monitoring Notice */
.monitoring-notice {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 16px 24px;
  background: #fef3c7;
  color: #92400e;
  font-size: 14px;
  border-top: 1px solid #e5e7eb;
}

.notice-icon {
  font-size: 16px;
}

.config-group {
  margin-bottom: 20px;
}

.config-group:last-child {
  margin-bottom: 0;
}

.config-label {
  display: block;
  font-weight: 600;
  color: #555;
  margin-bottom: 8px;
}

.text-input {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid #d1d5db;
  border-radius: 4px;
  font-size: 14px;
}

.text-input:disabled {
  background: #f3f4f6;
  color: #6b7280;
}

.core-selection {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  margin-bottom: 8px;
}

.core-checkbox {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 14px;
  cursor: pointer;
}

.core-checkbox input:disabled {
  cursor: not-allowed;
}

.affinity-info {
  font-size: 14px;
  color: #6b7280;
  display: flex;
  gap: 16px;
}

.priority-select {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid #d1d5db;
  border-radius: 4px;
  font-size: 14px;
  background: white;
}

.priority-select:disabled {
  background: #f3f4f6;
  color: #6b7280;
}

.priority-info {
  font-size: 14px;
  color: #6b7280;
  margin-top: 4px;
}

.preset-buttons {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.preset-btn {
  padding: 8px 12px;
  border: 1px solid #d1d5db;
  border-radius: 4px;
  background: white;
  cursor: pointer;
  font-size: 14px;
  transition: background-color 0.2s;
}

.preset-btn:hover:not(:disabled) {
  background: #f3f4f6;
}

.preset-btn:disabled {
  background: #f3f4f6;
  color: #6b7280;
  cursor: not-allowed;
}

.monitoring-warning {
  color: #f59e0b;
  font-weight: 600;
  margin-top: 12px;
}

.error-warning {
  color: #ef4444;
  font-weight: 600;
  margin-top: 12px;
}

.admin-tip {
  color: #f59e0b;
  margin-top: 12px;
}

/* Controls */
.controls {
  background: white;
  padding: 16px;
  border-radius: 8px;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

.control-buttons {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 12px;
}

.btn {
  padding: 10px 16px;
  border: none;
  border-radius: 4px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: background-color 0.2s;
}

.btn-start {
  background: #22c55e;
  color: white;
}

.btn-start:hover:not(:disabled) {
  background: #16a34a;
}

.btn-start:disabled {
  background: #d1d5db;
  color: #6b7280;
  cursor: not-allowed;
}

.btn-stop {
  background: #ef4444;
  color: white;
}

.btn-stop:hover {
  background: #dc2626;
}

.btn-secondary {
  background: #6b7280;
  color: white;
  padding: 8px 12px;
  font-size: 13px;
}

.btn-secondary:hover {
  background: #4b5563;
}

.status {
  font-weight: 600;
}

.status-active {
  color: #22c55e;
}

.status-inactive {
  color: #ef4444;
}

.secondary-controls {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 14px;
  cursor: pointer;
}

.log-count {
  font-size: 14px;
  color: #6b7280;
}

/* Tips */
.tips {
  background: #fef3c7;
  padding: 12px;
  border-radius: 4px;
  border-left: 4px solid #f59e0b;
}

.tip {
  color: #92400e;
  font-size: 14px;
}

/* Logs */
.logs-section {
  background: white;
  border-radius: 8px;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
  overflow: hidden;
}

.logs-section h2 {
  margin: 0;
  padding: 16px 20px;
  background: #f9fafb;
  border-bottom: 1px solid #e5e7eb;
  color: #333;
  font-size: 16px;
}

.logs-container {
  height: 200px;
  overflow-y: auto;
  padding: 12px;
}

.no-logs {
  color: #6b7280;
  text-align: center;
  padding: 20px;
}

.logs {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.log-entry {
  display: flex;
  gap: 8px;
  font-size: 13px;
  font-family: 'Courier New', monospace;
  padding: 4px 0;
}

.log-level {
  font-weight: 600;
  min-width: 80px;
}

.log-time {
  color: #6b7280;
  min-width: 80px;
}

.log-message {
  flex: 1;
}

.log-error .log-level {
  color: #ef4444;
}

.log-success .log-level {
  color: #22c55e;
}

.log-reapply .log-level {
  color: #f59e0b;
}

.log-monitor .log-level {
  color: #6b7280;
}

.log-info .log-level {
  color: #3b82f6;
}
</style>
