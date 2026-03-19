import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useEffect, useState, useCallback } from 'react';
import { Config, AppState, DEFAULT_STATE, DEFAULT_CONFIG } from '../types';

export function useTauri() {
  const [config,    setConfig]    = useState<Config>(DEFAULT_CONFIG);
  const [appState,  setAppState]  = useState<AppState>(DEFAULT_STATE);
  const [logs,      setLogs]      = useState<string[]>([]);
  const [running,   setRunning]   = useState(false);
  const [saving,    setSaving]    = useState(false);
  const [saveError, setSaveError] = useState('');

  useEffect(() => {
    invoke<Config>('get_config').then(setConfig).catch(console.error);
    invoke<AppState>('get_state').then(setAppState).catch(console.error);
    invoke<boolean>('get_running').then(setRunning).catch(console.error);
  }, []);

  useEffect(() => {
    const unsubs: Promise<() => void>[] = [];

    unsubs.push(listen<AppState>('state-update', e => {
      setAppState(e.payload);
      setRunning(e.payload.status === 'connected' || e.payload.status === 'connecting');
    }));

    unsubs.push(listen<string>('log', e => {
      setLogs(prev => {
        const next = [...prev, e.payload];
        return next.length > 200 ? next.slice(-200) : next;
      });
    }));

    return () => { unsubs.forEach(p => p.then(fn => fn())); };
  }, []);

  const saveConfig = useCallback(async (cfg: Config): Promise<boolean> => {
    setSaving(true);
    setSaveError('');
    try {
      await invoke('save_config', { cfg });
      setConfig(cfg);
      return true;
    } catch (e: unknown) {
      setSaveError(String(e));
      return false;
    } finally {
      setSaving(false);
    }
  }, []);

  const connect = useCallback(async (cfg: Config) => {
    const ok = await saveConfig(cfg);
    if (!ok) return;
    try {
      await invoke('connect');
    } catch (e: unknown) {
      setSaveError(String(e));
    }
  }, [saveConfig]);

  const disconnect = useCallback(async () => {
    await invoke('disconnect');
    setRunning(false);
  }, []);

  return {
    config, appState, logs, running, saving, saveError,
    saveConfig, connect, disconnect,
  };
}
