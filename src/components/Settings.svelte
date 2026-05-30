<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  interface Props {
    isOpen: boolean;
    onClose: () => void;
    presentMonApps: string[];
    availableSensors: Array<{ identifier: string; name: string }>;
    onSettingsChanged: (settings: any) => void;
  }

  let { 
    isOpen = false, 
    onClose, 
    presentMonApps = [], 
    availableSensors = [],
    onSettingsChanged
  }: Props = $props();

  // Active tab selection
  let activeTab: 'app' | 'style' | 'sensors' = $state('app');

  // Backend administrative states
  let isElevated = $state(false);
  let autostartEnabled = $state(false);
  
  // Settings model with precise matches to OverlaySettings.kt
  let settings = $state({
    isDarkTheme: false,
    isHorizontal: true,
    positionIndex: 0,
    selectedDisplayIndex: 0,
    netGraph: true,
    progressType: 'Circular' as 'Circular' | 'Bar' | 'None',
    positionX: 0,
    positionY: 0,
    isPositionLocked: false,
    opacity: 1.0,
    pollingRate: 500,
    isLoggingEnabled: false,
    sensors: {
      framerate: { isEnabled: true, customReadingId: '' },
      frametime: { isEnabled: true, customReadingId: '' },
      cpuTemp: { isEnabled: true, customReadingId: '/intelcpu/0/temperature/0', boundaries: { low: 60, medium: 80, high: 90 } },
      cpuUsage: { isEnabled: true, customReadingId: '/intelcpu/0/load/0', boundaries: { low: 60, medium: 80, high: 90 } },
      cpuConsumption: { isEnabled: true, customReadingId: '/intelcpu/0/power/0' },
      gpuTemp: { isEnabled: true, customReadingId: '/nvidiagpu/0/temperature/0', boundaries: { low: 60, medium: 80, high: 90 } },
      gpuUsage: { isEnabled: true, customReadingId: '/nvidiagpu/0/load/0', boundaries: { low: 60, medium: 80, high: 90 } },
      vramUsage: { isEnabled: true, customReadingId: '/nvidiagpu/0/load/3', boundaries: { low: 60, medium: 80, high: 90 } },
      totalVramUsed: { isEnabled: true, customReadingId: '/nvidiagpu/0/sm/0' },
      gpuConsumption: { isEnabled: true, customReadingId: '/nvidiagpu/0/power/0' },
      ramUsage: { isEnabled: true, customReadingId: '/ram/load/0', boundaries: { low: 60, medium: 80, high: 90 } },
      upRate: { isEnabled: true, customReadingId: '/nic/0/tx' },
      downRate: { isEnabled: true, customReadingId: '/nic/0/rx' },
    }
  });

  // Selected PresentMon target
  let selectedApp = $state('Auto');

  onMount(async () => {
    // 1. Fetch elevation and autostart statuses from Rust
    try {
      isElevated = await invoke('is_elevated');
      autostartEnabled = await invoke('is_autostart_enabled');
    } catch (e) {
      console.error(e);
    }

    // 2. Fetch preferences
    try {
      const stored: any = await invoke('get_settings');
      if (stored && stored.OVERLAY_SETTINGS_PREFERENCE_KEY) {
        const parsed = JSON.parse(stored.OVERLAY_SETTINGS_PREFERENCE_KEY);
        // Merge stored preferences with defaults
        settings = { ...settings, ...parsed };
      }
    } catch (e) {
      console.error("No stored settings, using default ones", e);
    }
  });

  // Save configurations reactively
  async function save() {
    try {
      const serialized = JSON.stringify(settings);
      const payload = {
        OVERLAY_SETTINGS_PREFERENCE_KEY: serialized,
        PREFERENCE_START_MINIMIZED: "false",
        PREFERENCE_PERMISSION_CONSENT: "true"
      };
      await invoke('save_settings', { settings: payload });
      onSettingsChanged(settings);
    } catch (e) {
      console.error("Failed to save settings", e);
    }
  }

  // Registry autostart toggle (HKCU)
  async function toggleAutostart() {
    try {
      autostartEnabled = !autostartEnabled;
      await invoke('set_autostart_enabled', { enabled: autostartEnabled });
    } catch (e) {
      console.error(e);
      autostartEnabled = !autostartEnabled;
    }
  }

  // Process elevation fallback
  async function triggerElevation() {
    await invoke('elevate');
  }

  // Windows Service Controls
  let serviceStatus = $state('');
  async function installService() {
    try {
      serviceStatus = "Installing service...";
      await invoke('install_service');
      serviceStatus = "Service installed successfully!";
    } catch (e: any) {
      serviceStatus = `Error: ${e}`;
    }
  }

  async function uninstallService() {
    try {
      serviceStatus = "Uninstalling service...";
      await invoke('uninstall_service');
      serviceStatus = "Service uninstalled successfully!";
    } catch (e: any) {
      serviceStatus = `Error: ${e}`;
    }
  }

  // Handle polling rate change
  async function changePollingRate(rate: number) {
    settings.pollingRate = rate;
    await save();
    // NotifyNamedPipe of new polling rate
    try {
      await invoke('select_polling_rate', { interval: rate });
    } catch (e) {
      console.error(e);
    }
  }

  // Handle PresentMon target focus change
  async function changePresentMonApp(app: string) {
    selectedApp = app;
    try {
      await invoke('select_present_mon_app', { name: app });
    } catch (e) {
      console.error(e);
    }
  }
</script>

{#if isOpen}
  <!-- Overlay Backdrop -->
  <div class="fixed inset-0 bg-black/60 backdrop-blur-sm z-50 flex items-center justify-center p-4">
    
    <!-- Modal Window (Glassmorphic Theme matching design system) -->
    <div class="w-full max-w-2xl bg-gray-900/90 border border-gray-800 rounded-2xl shadow-2xl overflow-hidden glass flex flex-col max-h-[85vh] transition-all duration-300">
      
      <!-- Top Bar -->
      <div class="flex items-center justify-between px-6 py-4 border-b border-gray-800/80">
        <h2 class="text-titleM font-medium text-white flex items-center gap-2 uppercase tracking-wider select-none">
          <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5 text-indicator-green" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.1a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/><circle cx="12" cy="12" r="3"/></svg>
          CleanMeter Settings
        </h2>
        <button onclick={onClose} class="text-gray-400 hover:text-white transition-colors p-1 rounded-lg hover:bg-gray-800">
          <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
        </button>
      </div>

      <!-- Tab Selection -->
      <div class="flex border-b border-gray-800/80 bg-gray-950/40 select-none">
        <button 
          onclick={() => activeTab = 'app'}
          class="flex-1 py-3 text-labelS font-medium transition-all border-b-2 uppercase tracking-wide
            {activeTab === 'app' ? 'text-indicator-green border-indicator-green bg-gray-900/10' : 'text-gray-400 border-transparent hover:text-white'}"
        >
          App Configuration
        </button>
        <button 
          onclick={() => activeTab = 'style'}
          class="flex-1 py-3 text-labelS font-medium transition-all border-b-2 uppercase tracking-wide
            {activeTab === 'style' ? 'text-indicator-green border-indicator-green bg-gray-900/10' : 'text-gray-400 border-transparent hover:text-white'}"
        >
          Theme & Overlay Style
        </button>
        <button 
          onclick={() => activeTab = 'sensors'}
          class="flex-1 py-3 text-labelS font-medium transition-all border-b-2 uppercase tracking-wide
            {activeTab === 'sensors' ? 'text-indicator-green border-indicator-green bg-gray-900/10' : 'text-gray-400 border-transparent hover:text-white'}"
        >
          Sensor Customization
        </button>
      </div>

      <!-- Content Area (Scrollable) -->
      <div class="flex-1 overflow-y-auto p-6 space-y-6">
        
        <!-- Tab 1: App Settings -->
        {#if activeTab === 'app'}
          <div class="space-y-6">
            <!-- Autostart (Non-elevated HKCU) -->
            <div class="flex items-center justify-between bg-gray-950/20 p-4 border border-gray-800/60 rounded-xl">
              <div>
                <h3 class="text-labelL font-medium text-white">Start with Windows</h3>
                <p class="text-labelS text-gray-400">Launch CleanMeter overlay automatically on user login.</p>
              </div>
              <button 
                onclick={toggleAutostart}
                class="w-12 h-6 rounded-full p-1 transition-colors duration-200 focus:outline-none
                  {autostartEnabled ? 'bg-indicator-green' : 'bg-gray-700'}"
              >
                <div class="w-4 h-4 rounded-full bg-white transition-transform duration-200
                  {autostartEnabled ? 'translate-x-6' : 'translate-x-0'}"></div>
              </button>
            </div>

            <!-- Privilege Elevation Management -->
            <div class="bg-gray-950/20 p-4 border border-gray-800/60 rounded-xl space-y-4">
              <div class="flex items-center justify-between">
                <div>
                  <h3 class="text-labelL font-medium text-white">Administrator Access</h3>
                  <p class="text-labelS text-gray-400">Required to manage svcleanmeter background poller service.</p>
                </div>
                {#if isElevated}
                  <span class="px-2 py-1 bg-indicator-green/20 text-indicator-green text-bodyM rounded font-medium border border-indicator-green/30 uppercase">
                    ELEVATED
                  </span>
                {:else}
                  <button 
                    onclick={triggerElevation}
                    class="px-3 py-1.5 bg-gray-800 hover:bg-gray-700 text-white rounded-lg text-labelS font-medium transition-colors border border-gray-700"
                  >
                    Run as Admin
                  </button>
                {/if}
              </div>

              <!-- Service Installer Panel (Disabled if non-elevated) -->
              <div class="pt-4 border-t border-gray-800/80 space-y-3">
                <h4 class="text-labelS font-semibold text-gray-300 uppercase tracking-wider">Windows Service Installer</h4>
                <div class="flex gap-3">
                  <button 
                    disabled={!isElevated}
                    onclick={installService}
                    class="px-4 py-2 rounded-lg text-labelS font-medium transition-all border
                      {isElevated ? 'bg-indicator-green/20 text-indicator-green border-indicator-green/30 hover:bg-indicator-green/30' : 'bg-gray-950/50 text-gray-600 border-gray-800/50 cursor-not-allowed'}"
                  >
                    Install svcleanmeter Service
                  </button>
                  <button 
                    disabled={!isElevated}
                    onclick={uninstallService}
                    class="px-4 py-2 rounded-lg text-labelS font-medium transition-all border
                      {isElevated ? 'bg-red-500/20 text-red-400 border-red-500/30 hover:bg-red-500/30' : 'bg-gray-950/50 text-gray-600 border-gray-800/50 cursor-not-allowed'}"
                  >
                    Uninstall Service
                  </button>
                </div>
                {#if serviceStatus}
                  <p class="text-bodyM font-medium text-indicator-yellow italic">{serviceStatus}</p>
                {/if}
              </div>
            </div>

            <!-- PresentMon Focus Tracking -->
            <div class="bg-gray-950/20 p-4 border border-gray-800/60 rounded-xl space-y-3">
              <div>
                <h3 class="text-labelL font-medium text-white">PresentMon App Focus</h3>
                <p class="text-labelS text-gray-400">Target game or application for real-time framerate telemetry.</p>
              </div>
              <select 
                value={selectedApp} 
                onchange={(e) => changePresentMonApp((e.target as HTMLSelectElement).value)}
                class="w-full bg-gray-950 border border-gray-800 rounded-lg p-2.5 text-labelS text-white focus:outline-none focus:border-indicator-green"
              >
                {#each presentMonApps as app}
                  <option value={app}>{app}</option>
                {/each}
              </select>
            </div>
          </div>
        {/if}

        <!-- Tab 2: Style Settings -->
        {#if activeTab === 'style'}
          <div class="space-y-6">
            <!-- Theme & Layout (Horizontal/Vertical) -->
            <div class="grid grid-cols-2 gap-4">
              <div class="bg-gray-950/20 p-4 border border-gray-800/60 rounded-xl">
                <h3 class="text-labelL font-medium text-white mb-3">Overlay Layout</h3>
                <div class="flex gap-2">
                  <button 
                    onclick={() => { settings.isHorizontal = true; save(); }}
                    class="flex-1 py-2 rounded-lg border text-labelS font-medium transition-all
                      {settings.isHorizontal ? 'bg-indicator-green/20 text-indicator-green border-indicator-green/30' : 'bg-gray-950 border-gray-800 text-gray-400 hover:text-white'}"
                  >
                    Horizontal
                  </button>
                  <button 
                    onclick={() => { settings.isHorizontal = false; save(); }}
                    class="flex-1 py-2 rounded-lg border text-labelS font-medium transition-all
                      {!settings.isHorizontal ? 'bg-indicator-green/20 text-indicator-green border-indicator-green/30' : 'bg-gray-950 border-gray-800 text-gray-400 hover:text-white'}"
                  >
                    Vertical
                  </button>
                </div>
              </div>

              <!-- Gauge Progress Styles -->
              <div class="bg-gray-950/20 p-4 border border-gray-800/60 rounded-xl">
                <h3 class="text-labelL font-medium text-white mb-3">Gauge Progress Mode</h3>
                <div class="flex gap-2">
                  {#each ['Circular', 'Bar', 'None'] as mode}
                    <button 
                      onclick={() => { settings.progressType = mode as any; save(); }}
                      class="flex-1 py-2 rounded-lg border text-labelS font-medium transition-all
                        {settings.progressType === mode ? 'bg-indicator-green/20 text-indicator-green border-indicator-green/30' : 'bg-gray-950 border-gray-800 text-gray-400 hover:text-white'}"
                    >
                      {mode}
                    </button>
                  {/each}
                </div>
              </div>
            </div>

            <!-- Network Speeds Chart Toggle -->
            <div class="flex items-center justify-between bg-gray-950/20 p-4 border border-gray-800/60 rounded-xl">
              <div>
                <h3 class="text-labelL font-medium text-white">Network Flows Graph</h3>
                <p class="text-labelS text-gray-400">Renders real-time download and upload line flows in the background.</p>
              </div>
              <button 
                onclick={() => { settings.netGraph = !settings.netGraph; save(); }}
                class="w-12 h-6 rounded-full p-1 transition-colors duration-200 focus:outline-none
                  {settings.netGraph ? 'bg-indicator-green' : 'bg-gray-700'}"
              >
                <div class="w-4 h-4 rounded-full bg-white transition-transform duration-200
                  {settings.netGraph ? 'translate-x-6' : 'translate-x-0'}"></div>
              </button>
            </div>

            <!-- Polling Interval Rate -->
            <div class="bg-gray-950/20 p-4 border border-gray-800/60 rounded-xl space-y-3">
              <div class="flex justify-between items-center">
                <div>
                  <h3 class="text-labelL font-medium text-white">Polling Interval Rate</h3>
                  <p class="text-labelS text-gray-400">Adjust how fast system monitoring data is polled and refreshed.</p>
                </div>
                <span class="text-labelL font-bold text-indicator-green">{settings.pollingRate}ms</span>
              </div>
              <select 
                value={settings.pollingRate} 
                onchange={(e) => changePollingRate(parseInt((e.target as HTMLSelectElement).value))}
                class="w-full bg-gray-950 border border-gray-800 rounded-lg p-2.5 text-labelS text-white focus:outline-none focus:border-indicator-green"
              >
                <option value={33}>33 ms (High Fidelity - 30Hz)</option>
                <option value={100}>100 ms</option>
                <option value={250}>250 ms</option>
                <option value={500}>500 ms (Recommended)</option>
                <option value={1000}>1000 ms</option>
                <option value={2000}>2000 ms</option>
              </select>
            </div>

            <!-- Opacity Slider -->
            <div class="bg-gray-950/20 p-4 border border-gray-800/60 rounded-xl space-y-3">
              <div class="flex justify-between items-center">
                <div>
                  <h3 class="text-labelL font-medium text-white">Overlay Opacity</h3>
                  <p class="text-labelS text-gray-400">Set the master transparency level of the overlay panels.</p>
                </div>
                <span class="text-labelL font-bold text-indicator-green">{Math.round(settings.opacity * 100)}%</span>
              </div>
              <input 
                type="range" 
                min="0.1" 
                max="1.0" 
                step="0.05"
                bind:value={settings.opacity}
                oninput={save}
                class="w-full accent-indicator-green"
              />
            </div>
          </div>
        {/if}

        <!-- Tab 3: Sensor Customization -->
        {#if activeTab === 'sensors'}
          <div class="space-y-4">
            <h3 class="text-labelL font-medium text-white uppercase tracking-wider text-gray-300">Sensor Mappings</h3>
            
            <div class="space-y-3">
              {#each Object.keys(settings.sensors) as key}
                {@const sensor = (settings.sensors as any)[key]}
                <div class="bg-gray-950/20 p-4 border border-gray-800/60 rounded-xl space-y-3">
                  <div class="flex items-center justify-between">
                    <span class="text-labelL font-medium text-white uppercase tracking-wide">
                      {key.replace(/([A-Z])/g, ' $1')}
                    </span>
                    <button 
                      onclick={() => { sensor.isEnabled = !sensor.isEnabled; save(); }}
                      class="w-12 h-6 rounded-full p-1 transition-colors duration-200 focus:outline-none
                        {sensor.isEnabled ? 'bg-indicator-green' : 'bg-gray-700'}"
                    >
                      <div class="w-4 h-4 rounded-full bg-white transition-transform duration-200
                        {sensor.isEnabled ? 'translate-x-6' : 'translate-x-0'}"></div>
                    </button>
                  </div>

                  {#if sensor.isEnabled}
                    <div class="space-y-3 pt-2 border-t border-gray-800/40">
                      <!-- Dropdown to pick mapped sensor ID -->
                      <div class="space-y-1">
                        <label class="text-bodyM text-gray-400 uppercase tracking-widest block">Custom Sensor Address</label>
                        <select 
                          bind:value={sensor.customReadingId}
                          onchange={save}
                          class="w-full bg-gray-950 border border-gray-800 rounded-lg p-2 text-labelS text-white focus:outline-none focus:border-indicator-green"
                        >
                          <option value="">-- Let Companion Select --</option>
                          {#each availableSensors as item}
                            <option value={item.identifier}>{item.name} ({item.identifier})</option>
                          {/each}
                        </select>
                      </div>

                      <!-- Bounds Configuration (If bounds exist) -->
                      {#if sensor.boundaries}
                        <div class="grid grid-cols-3 gap-2">
                          <div>
                            <label class="text-bodyM text-gray-400 uppercase tracking-widest block">Low (Green)</label>
                            <input 
                              type="number" 
                              bind:value={sensor.boundaries.low}
                              onchange={save}
                              class="w-full bg-gray-950 border border-gray-800 rounded-lg p-2 text-labelS text-white text-center focus:outline-none focus:border-indicator-green"
                            />
                          </div>
                          <div>
                            <label class="text-bodyM text-gray-400 uppercase tracking-widest block">Medium (Yellow)</label>
                            <input 
                              type="number" 
                              bind:value={sensor.boundaries.medium}
                              onchange={save}
                              class="w-full bg-gray-950 border border-gray-800 rounded-lg p-2 text-labelS text-white text-center focus:outline-none focus:border-indicator-green"
                            />
                          </div>
                          <div>
                            <label class="text-bodyM text-gray-400 uppercase tracking-widest block">High (Red)</label>
                            <input 
                              type="number" 
                              bind:value={sensor.boundaries.high}
                              onchange={save}
                              class="w-full bg-gray-950 border border-gray-800 rounded-lg p-2 text-labelS text-white text-center focus:outline-none focus:border-indicator-green"
                            />
                          </div>
                        </div>
                      {/if}
                    </div>
                  {/if}
                </div>
              {/each}
            </div>
          </div>
        {/if}

      </div>

      <!-- Footer Bar -->
      <div class="px-6 py-4 border-t border-gray-800/80 bg-gray-950/40 flex justify-end">
        <button 
          onclick={onClose}
          class="px-5 py-2 bg-indicator-green text-white font-medium text-labelS rounded-xl hover:bg-indicator-green/90 transition-all uppercase tracking-wider"
        >
          Done
        </button>
      </div>

    </div>
  </div>
{/if}
