<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { invoke } from '@tauri-apps/api/core';

  import Pill from '../components/Pill.svelte';
  import Progress from '../components/Progress.svelte';
  import FrametimeGraph from '../components/FrametimeGraph.svelte';
  import NetGraph from '../components/NetGraph.svelte';
  import Settings from '../components/Settings.svelte';

  // Named Pipe Telemetry Structures
  interface Hardware {
    name: string;
    identifier: string;
    hardware_type: number;
  }

  interface Sensor {
    name: string;
    identifier: string;
    hardware_identifier: string;
    sensor_type: number;
    value: number;
  }

  interface HardwareMonitorData {
    last_poll_time: number;
    hardwares: Hardware[];
    sensors: Sensor[];
    present_mon_apps: string[];
  }

  // Reactive state
  let data: HardwareMonitorData | null = $state(null);
  let isSettingsOpen = $state(false);
  let pipeStatus = $state('disconnected');

  // Load settings configuration
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
      cpuTemp: { isEnabled: true, customReadingId: '', boundaries: { low: 60, medium: 80, high: 90 } },
      cpuUsage: { isEnabled: true, customReadingId: '', boundaries: { low: 60, medium: 80, high: 90 } },
      cpuConsumption: { isEnabled: true, customReadingId: '' },
      gpuTemp: { isEnabled: true, customReadingId: '', boundaries: { low: 60, medium: 80, high: 90 } },
      gpuUsage: { isEnabled: true, customReadingId: '', boundaries: { low: 60, medium: 80, high: 90 } },
      vramUsage: { isEnabled: true, customReadingId: '', boundaries: { low: 60, medium: 80, high: 90 } },
      totalVramUsed: { isEnabled: true, customReadingId: '' },
      gpuConsumption: { isEnabled: true, customReadingId: '' },
      ramUsage: { isEnabled: true, customReadingId: '', boundaries: { low: 60, medium: 80, high: 90 } },
      upRate: { isEnabled: true, customReadingId: '' },
      downRate: { isEnabled: true, customReadingId: '' },
    }
  });

  // Hotkey listener and pipe unlisteners
  let unlistenPipe: () => void;
  let unlistenStatus: () => void;

  onMount(async () => {
    // 1. Launch C# sidecar companion poller
    try {
      await invoke('start_companion_process');
    } catch (e) {
      console.error("Companion sidecar launch failure:", e);
    }

    // 2. Fetch loaded overlay preferences
    try {
      const stored: any = await invoke('get_settings');
      if (stored && stored.OVERLAY_SETTINGS_PREFERENCE_KEY) {
        settings = { ...settings, ...JSON.parse(stored.OVERLAY_SETTINGS_PREFERENCE_KEY) };
      }
    } catch (e) {
      console.error(e);
    }

    // 3. Listen to named pipe telemetry event streams from Rust
    unlistenPipe = await listen<HardwareMonitorData>('hardware-data', (event) => {
      data = event.payload;
    });

    unlistenStatus = await listen<string>('pipe-status', (event) => {
      pipeStatus = event.payload;
    });

    // 4. Update click-through state based on position lock
    applyClickThrough();
  });

  onDestroy(() => {
    if (unlistenPipe) unlistenPipe();
    if (unlistenStatus) unlistenStatus();
    // Stop C# process on GUI exit
    invoke('stop_companion_process').catch(console.error);
  });

  // Apply click-through transparency
  async function applyClickThrough() {
    try {
      // If settings open, we must unlock window to allow mouse inputs!
      const shouldLock = settings.isPositionLocked && !isSettingsOpen;
      await invoke('set_window_click_through', { transparent: shouldLock });
    } catch (e) {
      console.error(e);
    }
  }

  function handleSettingsChanged(newSettings: any) {
    settings = newSettings;
    applyClickThrough();
  }

  function openSettings() {
    isSettingsOpen = true;
    applyClickThrough();
  }

  function closeSettings() {
    isSettingsOpen = false;
    applyClickThrough();
  }

  // Sensor address lookup helpers
  function getReading(customId: string, nameSearch: string): Sensor | null {
    if (!data) return null;
    
    // 1. Try search by exact custom ID mapping
    if (customId) {
      const match = data.sensors.find(s => s.identifier === customId);
      if (match) return match;
    }

    // 2. Fallback search by key substring matching Compose desktop models
    const lowerName = nameSearch.toLowerCase();
    const fallback = data.sensors.find(s => 
      s.identifier.toLowerCase().includes(lowerName) || 
      s.name.toLowerCase().includes(lowerName)
    );
    return fallback || null;
  }

  // Derived dynamic metrics
  let fps = $derived.by(() => {
    const reading = getReading(settings.sensors.framerate.customReadingId, '/presentmon/frametime');
    if (!reading || reading.value <= 0) return 0;
    return Math.round(1000 / reading.value);
  });

  let frametime = $derived.by(() => {
    const reading = getReading(settings.sensors.frametime.customReadingId, '/presentmon/frametime');
    return reading ? reading.value : 0;
  });

  let cpuTemp = $derived.by(() => {
    const reading = getReading(settings.sensors.cpuTemp.customReadingId, 'cpu temperature');
    return reading ? reading.value : 0;
  });

  let cpuUsage = $derived.by(() => {
    const reading = getReading(settings.sensors.cpuUsage.customReadingId, 'cpu total load');
    return reading ? reading.value : 0;
  });

  let cpuConsumption = $derived.by(() => {
    const reading = getReading(settings.sensors.cpuConsumption.customReadingId, 'cpu power');
    return reading ? Math.round(reading.value) : 0;
  });

  let gpuTemp = $derived.by(() => {
    const reading = getReading(settings.sensors.gpuTemp.customReadingId, 'gpu temperature');
    return reading ? reading.value : 0;
  });

  let gpuUsage = $derived.by(() => {
    const reading = getReading(settings.sensors.gpuUsage.customReadingId, 'gpu core load');
    return reading ? reading.value : 0;
  });

  let vramUsage = $derived.by(() => {
    const reading = getReading(settings.sensors.vramUsage.customReadingId, 'gpu memory load');
    return reading ? reading.value : 0;
  });

  let totalVramUsed = $derived.by(() => {
    const reading = getReading(settings.sensors.totalVramUsed.customReadingId, 'gpu memory dedicated');
    return reading ? reading.value : 0;
  });

  let gpuConsumption = $derived.by(() => {
    const reading = getReading(settings.sensors.gpuConsumption.customReadingId, 'gpu power');
    return reading ? Math.round(reading.value) : 0;
  });

  let ramUsagePercent = $derived.by(() => {
    if (!data) return 0;
    const used = data.sensors.find(s => s.name === 'Memory Used')?.value ?? 0;
    const avail = data.sensors.find(s => s.name === 'Memory Available')?.value ?? 0;
    return used + avail > 0 ? used / (used + avail) : 0;
  });

  let ramUsage = $derived.by(() => {
    if (!data) return 0;
    return data.sensors.find(s => s.name === 'Memory Used')?.value ?? 0;
  });

  let dlRate = $derived.by(() => {
    const reading = getReading(settings.sensors.downRate.customReadingId, 'network download');
    return reading ? reading.value : 0;
  });

  let upRate = $derived.by(() => {
    const reading = getReading(settings.sensors.upRate.customReadingId, 'network upload');
    return reading ? reading.value : 0;
  });

  // Extract all available sensors for the settings dropdown
  let availableSensorsList = $derived.by(() => {
    if (!data) return [];
    return data.sensors.map(s => ({
      identifier: s.identifier,
      name: `${s.name} [${s.hardware_identifier.replace('/intelcpu/0', 'CPU').replace('/nvidiagpu/0', 'GPU')}]`
    })).sort((a, b) => a.name.localeCompare(b.name));
  });

  // Check if network speeds should render
  let showNet = $derived(settings.sensors.upRate.isEnabled || settings.sensors.downRate.isEnabled);
</script>

<!-- Transparent Overlay Container -->
<div 
  class="relative h-screen flex select-none items-center justify-center p-4 transition-all duration-300"
  style="opacity: {settings.opacity};"
>
  <!-- Main Overlay Layout -->
  {#if data}
    <div 
      class="flex gap-2 glass p-1.5 transition-all duration-300
        {settings.isHorizontal ? 'flex-row rounded-full h-[60px] items-center' : 'flex-col rounded-xl w-[200px] h-auto'}"
    >
      <!-- CATEGORY 1: FPS & Frametime -->
      {#if settings.sensors.framerate.isEnabled || settings.sensors.frametime.isEnabled}
        <Pill title="FPS" isHorizontal={settings.isHorizontal}>
          {#if settings.sensors.framerate.isEnabled}
            <span class="text-titleM font-normal text-white select-none">{fps}</span>
          {/if}
          {#if settings.sensors.frametime.isEnabled}
            <FrametimeGraph frametime={frametime} isHorizontal={settings.isHorizontal} />
            <span class="text-labelS font-normal text-white/80 select-none">{frametime.toFixed(1)}ms</span>
          {/if}
        </Pill>
      {/if}

      <!-- CATEGORY 2: GPU -->
      {#if settings.sensors.gpuTemp.isEnabled || settings.sensors.gpuUsage.isEnabled || settings.sensors.vramUsage.isEnabled || settings.sensors.gpuConsumption.isEnabled}
        <Pill title="GPU" isHorizontal={settings.isHorizontal}>
          {#if settings.sensors.gpuTemp.isEnabled}
            <Progress 
              value={gpuTemp / 100} 
              label={Math.round(gpuTemp).toString()} 
              unit="°C" 
              progressType={settings.progressType} 
              boundaries={settings.sensors.gpuTemp.boundaries}
            />
          {/if}
          {#if settings.sensors.gpuUsage.isEnabled}
            <Progress 
              value={gpuUsage / 100} 
              label={Math.round(gpuUsage).toString()} 
              unit="%" 
              progressType={settings.progressType} 
              boundaries={settings.sensors.gpuUsage.boundaries}
            />
          {/if}
          {#if settings.sensors.vramUsage.isEnabled}
            <Progress 
              value={vramUsage / 100} 
              label={(totalVramUsed / 1024).toFixed(1)} 
              unit="GB" 
              progressType={settings.progressType} 
              boundaries={settings.sensors.vramUsage.boundaries}
            />
          {/if}
          {#if settings.sensors.gpuConsumption.isEnabled}
            <div class="flex items-end pb-[2px]">
              <span class="text-titleM font-normal text-white leading-none">{gpuConsumption}</span>
              <span class="text-bodyM font-normal text-white/70 pl-[1px] leading-none">W</span>
            </div>
          {/if}
        </Pill>
      {/if}

      <!-- CATEGORY 3: CPU -->
      {#if settings.sensors.cpuTemp.isEnabled || settings.sensors.cpuUsage.isEnabled || settings.sensors.cpuConsumption.isEnabled}
        <Pill title="CPU" isHorizontal={settings.isHorizontal}>
          {#if settings.sensors.cpuTemp.isEnabled}
            <Progress 
              value={cpuTemp / 100} 
              label={Math.round(cpuTemp).toString()} 
              unit="°C" 
              progressType={settings.progressType} 
              boundaries={settings.sensors.cpuTemp.boundaries}
            />
          {/if}
          {#if settings.sensors.cpuUsage.isEnabled}
            <Progress 
              value={cpuUsage / 100} 
              label={Math.round(cpuUsage).toString()} 
              unit="%" 
              progressType={settings.progressType} 
              boundaries={settings.sensors.cpuUsage.boundaries}
            />
          {/if}
          {#if settings.sensors.cpuConsumption.isEnabled}
            <div class="flex items-end pb-[2px]">
              <span class="text-titleM font-normal text-white leading-none">{cpuConsumption}</span>
              <span class="text-bodyM font-normal text-white/70 pl-[1px] leading-none">W</span>
            </div>
          {/if}
        </Pill>
      {/if}

      <!-- CATEGORY 4: RAM -->
      {#if settings.sensors.ramUsage.isEnabled}
        <Pill title="RAM" isHorizontal={settings.isHorizontal}>
          <Progress 
            value={ramUsagePercent} 
            label={ramUsage.toFixed(1)} 
            unit="GB" 
            progressType={settings.progressType} 
            boundaries={settings.sensors.ramUsage.boundaries}
          />
        </Pill>
      {/if}

      <!-- CATEGORY 5: NET -->
      {#if showNet}
        <Pill title="NET" isHorizontal={settings.isHorizontal}>
          <div class="flex flex-col justify-center gap-0.5 select-none text-[10px] uppercase font-semibold">
            {#if settings.sensors.downRate.isEnabled}
              <div class="flex items-center gap-1 text-indicator-cyan leading-none">
                <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="5" x2="12" y2="19"/><polyline points="19 12 12 19 5 12"/></svg>
                <span>{(dlRate / 1024 / 1024).toFixed(1)} M/s</span>
              </div>
            {/if}
            {#if settings.sensors.upRate.isEnabled}
              <div class="flex items-center gap-1 text-indicator-purple leading-none">
                <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="19" x2="12" y2="5"/><polyline points="5 12 12 5 19 12"/></svg>
                <span>{(upRate / 1024 / 1024).toFixed(1)} M/s</span>
              </div>
            {/if}
          </div>
          {#if settings.netGraph}
            <NetGraph 
              dlRate={dlRate} 
              upRate={upRate} 
              isHorizontal={settings.isHorizontal} 
              isDlEnabled={settings.sensors.downRate.isEnabled}
              isUpEnabled={settings.sensors.upRate.isEnabled}
            />
          {/if}
        </Pill>
      {/if}

      <!-- Settings Icon Toggler (Shows on Hover when Unlocked) -->
      <button 
        onclick={openSettings}
        class="p-2 text-white/50 hover:text-white transition-colors duration-200 cursor-pointer
          {settings.isHorizontal ? 'rounded-full hover:bg-white/10 ml-1' : 'w-full rounded-lg hover:bg-white/10 mt-1 flex justify-center'}"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>
      </button>

    </div>
  {:else}
    <!-- Seamless backdrop loading state (very premium) -->
    <div class="flex items-center gap-3 bg-black/40 backdrop-blur-md px-6 py-3 rounded-full border border-white/5 select-none">
      <div class="w-4 h-4 border-2 border-indicator-green border-t-transparent rounded-full animate-spin"></div>
      <span class="text-labelS font-medium text-white/90 tracking-wide">
        {#if pipeStatus === 'disconnected'}
          Waiting for companion service...
        {:else}
          Connecting to companion poller...
        {/if}
      </span>
    </div>
  {/if}

  <!-- Floating Settings Modal -->
  <Settings 
    isOpen={isSettingsOpen} 
    onClose={closeSettings}
    presentMonApps={data ? data.present_mon_apps : ['Auto']}
    availableSensors={availableSensorsList}
    onSettingsChanged={handleSettingsChanged}
  />
</div>
