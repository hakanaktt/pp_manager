<template>
  <div class="app">
    <div class="win9x-window">
      <div class="win9x-titlebar">
        <div class="win9x-titlebar-text">🎯 Process Priority Manager</div>
        <div class="win9x-titlebar-controls">
          <button class="win9x-titlebar-button" @click="minimizeToTray" title="Minimize">_</button>
          <button class="win9x-titlebar-button" title="Close">×</button>
        </div>
      </div>

      <div class="win9x-window-content">
        <!-- System Information -->
        <div class="win9x-groupbox">
          <div class="win9x-groupbox-title">System Information</div>
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
        </div>

      <!-- Tracked Processes Section -->
      <div v-if="trackedProcesses.length > 0" class="win9x-groupbox">
        <div class="win9x-groupbox-title">🎯 Tracked Process Instances</div>
        <div class="process-list">
          <div
            v-for="process in trackedProcesses"
            :key="process.pid"
            class="win9x-panel process-item enhanced"
            @mouseenter="loadProcessDetails(process.pid)"
          >
            <div class="process-header">
              <div class="process-info">
                <div class="process-main-info">
                  <span class="process-name">
                    {{ processDetails.get(process.pid)?.name || 'Loading...' }}
                  </span>
                  <span class="process-pid">PID: {{ process.pid }}</span>
                </div>
                <span class="process-status">🟢 Active</span>
              </div>
            </div>

            <div class="process-details-grid">
              <div class="detail-section">
                <h4>Applied Settings</h4>
                <div class="detail-items">
                  <span v-if="process.last_applied_affinity" class="process-detail">
                    <strong>Affinity:</strong> {{ formatHex(process.last_applied_affinity) }}
                  </span>
                  <span v-if="process.last_applied_priority" class="process-detail">
                    <strong>Priority:</strong> {{ formatHex(process.last_applied_priority) }}
                  </span>
                  <span v-if="!process.last_applied_affinity && !process.last_applied_priority" class="process-detail no-settings">
                    No settings applied yet
                  </span>
                </div>
              </div>

              <div v-if="processDetails.get(process.pid)" class="detail-section">
                <h4>Current State</h4>
                <div class="detail-items">
                  <span v-if="processDetails.get(process.pid)?.current_affinity" class="process-detail">
                    <strong>Affinity:</strong> {{ formatHex(processDetails.get(process.pid)!.current_affinity!) }}
                  </span>
                  <span v-if="processDetails.get(process.pid)?.current_priority" class="process-detail">
                    <strong>Priority:</strong> {{ formatHex(processDetails.get(process.pid)!.current_priority!) }}
                  </span>
                </div>
              </div>
            </div>

            <div class="process-actions">
              <button
                class="win9x-button"
                @click="showProcessDetails(process)"
                title="Show detailed process information"
              >
                📊 Details
              </button>
              <button
                class="win9x-button"
                @click="excludeProcess(process.pid)"
                title="Exclude this process from monitoring"
                :disabled="isMonitoring"
              >
                ❌ Exclude
              </button>
              <button
                class="win9x-button"
                @click="killProcess(process.pid)"
                title="Terminate this process (WARNING: This will forcefully close the process)"
                :disabled="isMonitoring"
              >
                💀 Kill
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
            class="win9x-button"
          >
            🔄 Show All
          </button>
        </div>
      </div>



      <!-- Multi-Process Management Section -->
      <div class="win9x-groupbox">
        <div class="win9x-groupbox-title">🎯 Process Management</div>
        <div class="section-subtitle">Configure multiple processes with individual CPU and priority settings</div>
        
        <div style="margin: 8px 0;">
          <button
            @click="showAddProcess = true"
            :disabled="isMonitoring"
            class="win9x-button"
          >
            + Add Process
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
      </div>

      <!-- Control Buttons -->
      <div class="win9x-groupbox">
        <div class="win9x-groupbox-title">Controls</div>
        <div class="control-buttons" style="display: flex; gap: 8px; margin: 8px 0;">
          <button 
            v-if="isMonitoring"
            @click="stopMonitoring"
            class="win9x-button"
          >
            ⏹ Stop Monitoring
          </button>
          <button 
            v-else
            :disabled="!canStartMonitoring"
            @click="startMonitoring"
            class="win9x-button-default"
          >
            ▶ Start Monitoring
          </button>
          
          <span :class="['status', isMonitoring ? 'status-active' : 'status-inactive']">
            {{ isMonitoring ? '🟢 Monitoring Active' : '🔴 Monitoring Stopped' }}
          </span>
        </div>

        <div class="secondary-controls" style="display: flex; gap: 8px; margin: 8px 0;">
          <button @click="clearLogs" class="win9x-button">🗑 Clear Logs</button>
          <button @click="minimizeToTray" class="win9x-button">📱 Minimize to Tray</button>
          <label style="display: flex; align-items: center; gap: 4px;">
            <input v-model="autoScroll" type="checkbox" class="win9x-checkbox" />
            Auto-scroll logs
          </label>
          <span style="color: #606060; font-size: 10px;">📊 {{ logs.length }} log entries</span>
        </div>
      </div>
        </div>
      </div>

      <!-- Tips -->
      <div class="win9x-groupbox">
        <div class="win9x-groupbox-title">💡 Tips</div>
        <div class="tip">
          Use Ctrl+Shift+P to show/hide this window, or use the system tray icon.
        </div>
      </div>

      <!-- Logs Section -->
      <div class="win9x-groupbox">
        <div class="win9x-groupbox-title">📋 Activity Logs</div>
        <div class="win9x-listbox logs-container" ref="logsContainer">
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
      </div>
    </div>
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

interface ProcessDetails {
  pid: number
  name: string
  current_priority: number | null
  current_affinity: number | null
  last_applied_priority: number | null
  last_applied_affinity: number | null
  is_tracked: boolean
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
const processDetails = ref<Map<number, ProcessDetails>>(new Map())
const loadingProcessDetails = ref<Set<number>>(new Set())

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
    const filteredProcesses = allProcesses.filter(p => !excludedProcesses.value.has(p.pid))
    trackedProcesses.value = filteredProcesses

    // Load details for new processes
    for (const process of filteredProcesses) {
      if (!processDetails.value.has(process.pid)) {
        loadProcessDetails(process.pid)
      }
    }

    // Clean up details for processes that are no longer tracked
    const currentPids = new Set(filteredProcesses.map(p => p.pid))
    for (const [pid] of processDetails.value) {
      if (!currentPids.has(pid)) {
        processDetails.value.delete(pid)
      }
    }
  } catch (error) {
    console.error('Failed to load tracked processes:', error)
  }
}

const loadProcessDetails = async (pid: number) => {
  if (loadingProcessDetails.value.has(pid)) return

  loadingProcessDetails.value.add(pid)
  try {
    const details = await invoke('get_process_details', { pid }) as ProcessDetails
    processDetails.value.set(pid, details)
  } catch (error) {
    console.error(`Failed to load details for PID ${pid}:`, error)
  } finally {
    loadingProcessDetails.value.delete(pid)
  }
}

const showProcessDetails = async (process: TrackedProcess) => {
  await loadProcessDetails(process.pid)
  const details = processDetails.value.get(process.pid)

  if (details) {
    const detailsText = [
      `Process Name: ${details.name}`,
      `Process ID: ${details.pid}`,
      `Current Priority: ${details.current_priority ? formatHex(details.current_priority) : 'Unknown'}`,
      `Current Affinity: ${details.current_affinity ? formatHex(details.current_affinity) : 'Unknown'}`,
      `Last Applied Priority: ${details.last_applied_priority ? formatHex(details.last_applied_priority) : 'None'}`,
      `Last Applied Affinity: ${details.last_applied_affinity ? formatHex(details.last_applied_affinity) : 'None'}`,
      `Tracking Status: ${details.is_tracked ? 'Tracked' : 'Not Tracked'}`
    ].join('\n')

    alert(`Process Details:\n\n${detailsText}`)
  } else {
    const basicDetails = [
      `Process ID: ${process.pid}`,
      `Last Applied Affinity: ${process.last_applied_affinity ? formatHex(process.last_applied_affinity) : 'None'}`,
      `Last Applied Priority: ${process.last_applied_priority ? formatHex(process.last_applied_priority) : 'None'}`
    ].join('\n')

    alert(`Process Details:\n\n${basicDetails}`)
  }
}

const killProcess = async (pid: number) => {
  const details = processDetails.value.get(pid)
  const processName = details?.name || `PID ${pid}`

  if (confirm(`Are you sure you want to terminate the process "${processName}"?\n\nThis action cannot be undone and may cause data loss if the process has unsaved work.`)) {
    try {
      await invoke('kill_process', { pid })
      // Remove from tracked processes and details
      trackedProcesses.value = trackedProcesses.value.filter(p => p.pid !== pid)
      processDetails.value.delete(pid)
      alert(`Process "${processName}" has been terminated successfully.`)
    } catch (error) {
      alert(`Failed to terminate process "${processName}": ${error}`)
    }
  }
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
/* Windows Aero Theme for Process Priority Manager */
.app {
  font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
  max-width: 100%;
  margin: 0;
  padding: 20px;
  min-height: 100vh;
  box-sizing: border-box;
  background: transparent;
}

.header {
  text-align: center;
  margin-bottom: 24px;
  padding: 20px;
  background: rgba(255, 255, 255, 0.95);
  border: 1px solid rgba(255, 255, 255, 0.4);
  border-radius: 12px;
  box-shadow:
    0 8px 32px rgba(0, 0, 0, 0.1),
    inset 0 1px 0 rgba(255, 255, 255, 0.6);
}

.header h1 {
  margin: 0;
  color: #2d3436;
  font-size: 28px;
  font-weight: 300;
  text-shadow: 0 1px 2px rgba(255, 255, 255, 0.8);
  background: linear-gradient(135deg, #74b9ff 0%, #0984e3 50%, #6c5ce7 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.main {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

/* System Info - Windows Aero Style */
.system-info {
  background: rgba(255, 255, 255, 0.9);
  padding: 20px;
  border-radius: 12px;
  border: 1px solid rgba(255, 255, 255, 0.4);
  box-shadow:
    0 8px 32px rgba(0, 0, 0, 0.1),
    inset 0 1px 0 rgba(255, 255, 255, 0.5);
  transition: all 0.3s ease;
}

.system-info:hover {
  box-shadow:
    0 12px 40px rgba(0, 0, 0, 0.15),
    inset 0 1px 0 rgba(255, 255, 255, 0.6);
  transform: translateY(-2px);
}

/* Tracked Processes - Windows Aero Style */
.tracked-processes {
  background: rgba(255, 255, 255, 0.9);
  padding: 20px;
  border-radius: 12px;
  border: 1px solid rgba(255, 255, 255, 0.4);
  box-shadow:
    0 8px 32px rgba(0, 0, 0, 0.1),
    inset 0 1px 0 rgba(255, 255, 255, 0.5);
  transition: all 0.3s ease;
}

.tracked-processes:hover {
  box-shadow:
    0 12px 40px rgba(0, 0, 0, 0.15),
    inset 0 1px 0 rgba(255, 255, 255, 0.6);
  transform: translateY(-2px);
}

.tracked-processes h2 {
  margin: 0 0 20px 0;
  color: #2d3436;
  font-size: 20px;
  font-weight: 300;
  text-shadow: 0 1px 2px rgba(255, 255, 255, 0.8);
}

.process-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
  margin-bottom: 16px;
}

.process-item {
  background: rgba(255, 255, 255, 0.8);
  padding: 16px;
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.3);
  border-left: 4px solid #00b894;
  box-shadow:
    0 4px 16px rgba(0, 0, 0, 0.08),
    inset 0 1px 0 rgba(255, 255, 255, 0.4);
  transition: all 0.3s ease;
}

.process-item:hover {
  background: rgba(255, 255, 255, 0.95);
  box-shadow:
    0 6px 24px rgba(0, 0, 0, 0.12),
    inset 0 1px 0 rgba(255, 255, 255, 0.5);
  transform: translateY(-1px);
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
  padding: 6px 12px;
  border: 1px solid #7fb3d3;
  border-radius: 4px;
  background: linear-gradient(180deg, #ffffff 0%, #e8f4fd 50%, #d0e8f7 100%);
  cursor: pointer;
  font-size: 11px;
  font-family: 'Segoe UI', sans-serif;
  color: #2d3436;
  transition: all 0.15s ease;
  box-shadow:
    0 1px 3px rgba(0, 0, 0, 0.1),
    inset 0 1px 0 rgba(255, 255, 255, 0.7);
}

.btn-process-action:hover:not(:disabled) {
  background: linear-gradient(180deg, #ffffff 0%, #f0f8ff 50%, #e0f0ff 100%);
  border-color: #5a9fd4;
  box-shadow:
    0 2px 6px rgba(0, 0, 0, 0.15),
    inset 0 1px 0 rgba(255, 255, 255, 0.8);
}

.btn-process-action:active:not(:disabled) {
  background: linear-gradient(180deg, #d0e8f7 0%, #b8ddf0 50%, #a0d2e9 100%);
  border-color: #4a8bc2;
  box-shadow:
    inset 0 1px 3px rgba(0, 0, 0, 0.2),
    0 1px 2px rgba(0, 0, 0, 0.1);
}

.btn-process-action:disabled {
  background: linear-gradient(180deg, #f5f5f5 0%, #e8e8e8 100%);
  border-color: #c0c0c0;
  color: #999;
  cursor: not-allowed;
  box-shadow: none;
}

.btn-exclude {
  color: #ef4444;
  border-color: #ef4444;
}

.btn-exclude:hover:not(:disabled) {
  background: #fef2f2;
}

.btn-kill {
  color: #dc2626;
  border-color: #dc2626;
  font-weight: 600;
}

.btn-kill:hover:not(:disabled) {
  background: #fef2f2;
  border-color: #b91c1c;
  color: #b91c1c;
}

/* Enhanced process item styles */
.process-item.enhanced {
  padding: 20px;
  background: rgba(255, 255, 255, 0.9);
  border-left: 4px solid #3b82f6;
}

.process-header {
  margin-bottom: 16px;
}

.process-main-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.process-name {
  font-weight: 700;
  color: #1f2937;
  font-size: 16px;
}

.process-details-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
  margin-bottom: 16px;
  padding: 12px;
  background: rgba(248, 250, 252, 0.8);
  border-radius: 6px;
  border: 1px solid rgba(226, 232, 240, 0.6);
}

.detail-section h4 {
  margin: 0 0 8px 0;
  font-size: 12px;
  font-weight: 600;
  color: #64748b;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.detail-items {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.process-detail {
  font-size: 13px;
  color: #475569;
  font-family: 'Courier New', monospace;
}

.process-detail strong {
  color: #334155;
  font-family: inherit;
}

.no-settings {
  color: #94a3b8;
  font-style: italic;
  font-family: inherit;
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



/* Windows Aero Process Management Section */
.processes-section {
  background: rgba(255, 255, 255, 0.9);
  border-radius: 16px;
  border: 1px solid rgba(255, 255, 255, 0.4);
  box-shadow:
    0 8px 32px rgba(0, 0, 0, 0.1),
    inset 0 1px 0 rgba(255, 255, 255, 0.5);
  overflow: hidden;
  margin-top: 20px;
  transition: all 0.3s ease;
}

.processes-section:hover {
  box-shadow:
    0 12px 40px rgba(0, 0, 0, 0.15),
    inset 0 1px 0 rgba(255, 255, 255, 0.6);
  transform: translateY(-2px);
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  padding: 28px;
  background: linear-gradient(135deg, #74b9ff 0%, #0984e3 50%, #6c5ce7 100%);
  color: white;
  position: relative;
  overflow: hidden;
}

.section-header::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(255, 255, 255, 0.1);
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
  gap: 10px;
  background: rgba(255, 255, 255, 0.25);
  color: white;
  border: 1px solid rgba(255, 255, 255, 0.4);
  padding: 12px 20px;
  border-radius: 8px;
  cursor: pointer;
  font-weight: 500;
  font-family: 'Segoe UI', sans-serif;
  transition: all 0.3s ease;
  position: relative;
  z-index: 1;
  box-shadow:
    0 4px 16px rgba(0, 0, 0, 0.1),
    inset 0 1px 0 rgba(255, 255, 255, 0.3);
}

.add-process-btn:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.35);
  border-color: rgba(255, 255, 255, 0.5);
  transform: translateY(-2px);
  box-shadow:
    0 6px 24px rgba(0, 0, 0, 0.15),
    inset 0 1px 0 rgba(255, 255, 255, 0.4);
}

.add-process-btn:active:not(:disabled) {
  transform: translateY(-1px);
  box-shadow:
    0 4px 16px rgba(0, 0, 0, 0.1),
    inset 0 1px 0 rgba(255, 255, 255, 0.3);
}

.add-process-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none;
}

.btn-icon {
  font-size: 16px;
  font-weight: bold;
}

/* Windows Aero Modal */
.add-process-modal {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  animation: modalFadeIn 0.3s ease;
}

@keyframes modalFadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

.modal-content {
  background: rgba(255, 255, 255, 0.95);
  border: 1px solid rgba(255, 255, 255, 0.4);
  border-radius: 16px;
  width: 90%;
  max-width: 520px;
  box-shadow:
    0 24px 48px rgba(0, 0, 0, 0.2),
    inset 0 1px 0 rgba(255, 255, 255, 0.6);
  animation: modalSlideIn 0.3s ease;
}

@keyframes modalSlideIn {
  from {
    opacity: 0;
    transform: translateY(-20px) scale(0.95);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
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
  border: 1px solid #7fb3d3;
  border-radius: 6px;
  font-size: 14px;
  font-family: 'Segoe UI', sans-serif;
  background: rgba(255, 255, 255, 0.95);
  color: #2d3436;
  transition: all 0.15s ease;
  box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.1);
}

.modern-input:focus {
  outline: none;
  border-color: #4a8bc2;
  background: rgba(255, 255, 255, 1);
  box-shadow:
    inset 0 1px 2px rgba(0, 0, 0, 0.1),
    0 0 8px rgba(74, 139, 194, 0.3);
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
  background: linear-gradient(180deg, #74b9ff 0%, #0984e3 50%, #0066cc 100%);
  color: white;
  padding: 10px 20px;
  border: 1px solid #0066cc;
  border-radius: 6px;
  cursor: pointer;
  font-weight: 500;
  font-family: 'Segoe UI', sans-serif;
  transition: all 0.15s ease;
  box-shadow:
    0 2px 6px rgba(0, 0, 0, 0.15),
    inset 0 1px 0 rgba(255, 255, 255, 0.3);
}

.btn-primary:hover {
  background: linear-gradient(180deg, #81c7ff 0%, #1e90ff 50%, #0080ff 100%);
  border-color: #0080ff;
  box-shadow:
    0 3px 8px rgba(0, 0, 0, 0.2),
    inset 0 1px 0 rgba(255, 255, 255, 0.4);
  transform: translateY(-1px);
}

.btn-primary:active {
  background: linear-gradient(180deg, #5a9fd4 0%, #0066cc 50%, #004499 100%);
  border-color: #004499;
  box-shadow:
    inset 0 1px 3px rgba(0, 0, 0, 0.3),
    0 1px 2px rgba(0, 0, 0, 0.1);
  transform: translateY(0);
}

.btn-secondary {
  background: linear-gradient(180deg, #ffffff 0%, #f8f9fa 50%, #e9ecef 100%);
  color: #495057;
  padding: 10px 20px;
  border: 1px solid #ced4da;
  border-radius: 6px;
  cursor: pointer;
  font-weight: 500;
  font-family: 'Segoe UI', sans-serif;
  transition: all 0.15s ease;
  box-shadow:
    0 1px 3px rgba(0, 0, 0, 0.1),
    inset 0 1px 0 rgba(255, 255, 255, 0.7);
}

.btn-secondary:hover {
  background: linear-gradient(180deg, #ffffff 0%, #f1f3f4 50%, #dee2e6 100%);
  border-color: #adb5bd;
  box-shadow:
    0 2px 6px rgba(0, 0, 0, 0.15),
    inset 0 1px 0 rgba(255, 255, 255, 0.8);
  transform: translateY(-1px);
}

.btn-secondary:active {
  background: linear-gradient(180deg, #e9ecef 0%, #dee2e6 50%, #ced4da 100%);
  border-color: #adb5bd;
  box-shadow:
    inset 0 1px 3px rgba(0, 0, 0, 0.2),
    0 1px 2px rgba(0, 0, 0, 0.1);
  transform: translateY(0);
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

/* Windows Aero Controls */
.controls {
  background: rgba(255, 255, 255, 0.9);
  padding: 20px;
  border-radius: 12px;
  border: 1px solid rgba(255, 255, 255, 0.4);
  box-shadow:
    0 8px 32px rgba(0, 0, 0, 0.1),
    inset 0 1px 0 rgba(255, 255, 255, 0.5);
  transition: all 0.3s ease;
}

.controls:hover {
  box-shadow:
    0 12px 40px rgba(0, 0, 0, 0.15),
    inset 0 1px 0 rgba(255, 255, 255, 0.6);
  transform: translateY(-2px);
}

.control-buttons {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 12px;
}

.btn {
  padding: 12px 20px;
  border: 1px solid;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  font-family: 'Segoe UI', sans-serif;
  cursor: pointer;
  transition: all 0.15s ease;
  box-shadow:
    0 2px 6px rgba(0, 0, 0, 0.15),
    inset 0 1px 0 rgba(255, 255, 255, 0.3);
}

.btn-start {
  background: linear-gradient(180deg, #00b894 0%, #00a085 50%, #008f72 100%);
  border-color: #008f72;
  color: white;
}

.btn-start:hover:not(:disabled) {
  background: linear-gradient(180deg, #00d2a4 0%, #00b894 50%, #00a085 100%);
  border-color: #00a085;
  box-shadow:
    0 3px 8px rgba(0, 0, 0, 0.2),
    inset 0 1px 0 rgba(255, 255, 255, 0.4);
  transform: translateY(-1px);
}

.btn-start:active:not(:disabled) {
  background: linear-gradient(180deg, #008f72 0%, #007d63 50%, #006b54 100%);
  border-color: #006b54;
  box-shadow:
    inset 0 1px 3px rgba(0, 0, 0, 0.3),
    0 1px 2px rgba(0, 0, 0, 0.1);
  transform: translateY(0);
}

.btn-start:disabled {
  background: linear-gradient(180deg, #f5f5f5 0%, #e8e8e8 100%);
  border-color: #c0c0c0;
  color: #999;
  cursor: not-allowed;
  box-shadow: none;
  transform: none;
}

.btn-stop {
  background: linear-gradient(180deg, #e17055 0%, #d63031 50%, #c0392b 100%);
  border-color: #c0392b;
  color: white;
}

.btn-stop:hover {
  background: linear-gradient(180deg, #fd79a8 0%, #e84393 50%, #d63031 100%);
  border-color: #d63031;
  box-shadow:
    0 3px 8px rgba(0, 0, 0, 0.2),
    inset 0 1px 0 rgba(255, 255, 255, 0.4);
  transform: translateY(-1px);
}

.btn-stop:active {
  background: linear-gradient(180deg, #c0392b 0%, #a93226 50%, #922b21 100%);
  border-color: #922b21;
  box-shadow:
    inset 0 1px 3px rgba(0, 0, 0, 0.3),
    0 1px 2px rgba(0, 0, 0, 0.1);
  transform: translateY(0);
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

/* Windows Aero Tips */
.tips {
  background: rgba(255, 243, 199, 0.9);
  padding: 16px;
  border-radius: 8px;
  border: 1px solid rgba(245, 158, 11, 0.3);
  border-left: 4px solid #f59e0b;
  box-shadow:
    0 4px 16px rgba(245, 158, 11, 0.1),
    inset 0 1px 0 rgba(255, 255, 255, 0.4);
}

.tip {
  color: #92400e;
  font-size: 14px;
  font-family: 'Segoe UI', sans-serif;
}

/* Windows Aero Logs */
.logs-section {
  background: rgba(255, 255, 255, 0.9);
  border-radius: 12px;
  border: 1px solid rgba(255, 255, 255, 0.4);
  box-shadow:
    0 8px 32px rgba(0, 0, 0, 0.1),
    inset 0 1px 0 rgba(255, 255, 255, 0.5);
  overflow: hidden;
  transition: all 0.3s ease;
}

.logs-section:hover {
  box-shadow:
    0 12px 40px rgba(0, 0, 0, 0.15),
    inset 0 1px 0 rgba(255, 255, 255, 0.6);
  transform: translateY(-2px);
}

.logs-section h2 {
  margin: 0;
  padding: 20px 24px;
  background: linear-gradient(135deg, rgba(116, 185, 255, 0.1) 0%, rgba(108, 92, 231, 0.1) 100%);
  border-bottom: 1px solid rgba(255, 255, 255, 0.3);
  color: #2d3436;
  font-size: 18px;
  font-weight: 300;
  text-shadow: 0 1px 2px rgba(255, 255, 255, 0.8);
}

.logs-container {
  height: 240px;
  overflow-y: auto;
  padding: 16px 20px;
  background: rgba(248, 249, 250, 0.5);
}

.logs-container::-webkit-scrollbar {
  width: 8px;
}

.logs-container::-webkit-scrollbar-track {
  background: rgba(255, 255, 255, 0.2);
  border-radius: 4px;
}

.logs-container::-webkit-scrollbar-thumb {
  background: linear-gradient(180deg, #74b9ff 0%, #0984e3 100%);
  border-radius: 4px;
  border: 1px solid rgba(255, 255, 255, 0.3);
}

.logs-container::-webkit-scrollbar-thumb:hover {
  background: linear-gradient(180deg, #81c7ff 0%, #1e90ff 100%);
}

.no-logs {
  color: #6b7280;
  text-align: center;
  padding: 32px 20px;
  font-style: italic;
}

.logs {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.log-entry {
  display: flex;
  gap: 12px;
  font-size: 12px;
  font-family: 'Consolas', 'Courier New', monospace;
  padding: 6px 12px;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.6);
  border: 1px solid rgba(255, 255, 255, 0.3);
  transition: all 0.2s ease;
}

.log-entry:hover {
  background: rgba(255, 255, 255, 0.8);
  border-color: rgba(255, 255, 255, 0.5);
  transform: translateX(2px);
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
