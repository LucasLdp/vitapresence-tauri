import { useState, useRef, useEffect, useCallback } from 'react';
import { useTauri } from './hooks/useTauri';
import { Config, DEFAULT_CONFIG, Lang, T } from './types';

// ── i18n hook ──────────────────────────────────────────────────────────
function useT(lang: Lang) {
  return useCallback((key: string) => T[lang][key] ?? key, [lang]);
}

// ── Components ─────────────────────────────────────────────────────────

function Toggle({ id, checked, onChange, label }: {
  id: string; checked: boolean; onChange: (v: boolean) => void; label: string;
}) {
  return (
    <label htmlFor={id} className="flex items-center gap-2.5 cursor-pointer group">
      <div className="relative flex-shrink-0">
        <input id={id} type="checkbox" className="sr-only peer" checked={checked}
          onChange={e => onChange(e.target.checked)}/>
        <div className="w-9 h-5 rounded-full border transition-all duration-200
          border-[#2a2a3d] bg-[#1a1a28]
          peer-checked:bg-[#0050c8] peer-checked:border-[#00a8ff]/40"/>
        <div className="absolute top-0.5 left-0.5 w-4 h-4 rounded-full shadow
          transition-all duration-200 bg-zinc-600
          peer-checked:translate-x-4 peer-checked:bg-white
          peer-checked:shadow-[0_0_6px_rgba(0,168,255,0.5)]"/>
      </div>
      <span className="text-xs text-zinc-500 group-hover:text-zinc-300 transition-colors">{label}</span>
    </label>
  );
}

function LangSwitch({ lang, onChange }: { lang: Lang; onChange: (l: Lang) => void }) {
  return (
    <div className="flex items-center gap-1 bg-[#0a0a0f] border border-[#2a2a3d] rounded-full p-0.5">
      <button
        onClick={() => onChange('pt')}
        className={`flex items-center gap-1 px-2.5 py-1 rounded-full text-[11px] font-semibold
          transition-all duration-200 ${
          lang === 'pt'
            ? 'bg-[#0050c8] text-white shadow-[0_0_8px_rgba(0,80,200,0.4)]'
            : 'text-zinc-500 hover:text-zinc-300'
        }`}
      >
        🇧🇷 PT
      </button>
      <button
        onClick={() => onChange('en')}
        className={`flex items-center gap-1 px-2.5 py-1 rounded-full text-[11px] font-semibold
          transition-all duration-200 ${
          lang === 'en'
            ? 'bg-[#0050c8] text-white shadow-[0_0_8px_rgba(0,80,200,0.4)]'
            : 'text-zinc-500 hover:text-zinc-300'
        }`}
      >
        🇺🇸 EN
      </button>
    </div>
  );
}

function StatusDot({ status }: { status: string }) {
  if (status === 'connected')
    return <span className="w-2.5 h-2.5 rounded-full bg-green-400 shadow-[0_0_8px_#4ade80] flex-shrink-0"/>;
  if (status === 'connecting')
    return <span className="w-2.5 h-2.5 rounded-full bg-orange-400 animate-pulse flex-shrink-0"/>;
  if (status === 'error')
    return <span className="w-2.5 h-2.5 rounded-full bg-red-500 flex-shrink-0"/>;
  return <span className="w-2.5 h-2.5 rounded-full bg-zinc-600 flex-shrink-0"/>;
}

function PlatformBadge({ platform, t }: { platform: string; t: (k:string)=>string }) {
  const map: Record<string,[string,string]> = {
    vita:     [t('platVita'),     'text-[#00a8ff] border-[#00a8ff]/30 bg-[#00a8ff]/10'],
    psp:      [t('platPsp'),      'text-yellow-400 border-yellow-400/30 bg-yellow-400/10'],
    emulator: [t('platEmulator'), 'text-green-400 border-green-400/30 bg-green-400/10'],
  };
  const e = map[platform];
  if (!e) return null;
  return (
    <span className={`text-[9px] font-bold uppercase tracking-widest px-2 py-0.5 rounded-full border ${e[1]}`}>
      {e[0]}
    </span>
  );
}

function Input({ error, className, ...props }: React.InputHTMLAttributes<HTMLInputElement> & { error?: boolean }) {
  return (
    <input {...props}
      className={`w-full bg-[#0a0a0f] border rounded-md px-3 py-2 text-sm text-zinc-200
        outline-none transition-all placeholder:text-zinc-700 font-mono
        ${error ? 'border-red-500' : 'border-[#2a2a3d] focus:border-[#00a8ff] focus:shadow-[0_0_0_2px_rgba(0,168,255,0.12)]'}
        ${className ?? ''}`}
    />
  );
}

function Field({ label, optional, hint, children }: {
  label: string; optional?: string; hint?: string; children: React.ReactNode;
}) {
  return (
    <div className="flex flex-col gap-1.5">
      <label className="text-[10px] font-semibold uppercase tracking-wider text-zinc-500 flex items-center gap-2">
        {label}
        {optional && (
          <span className="text-[9px] border border-[#2a2a3d] rounded px-1.5 py-px text-zinc-700 normal-case tracking-normal">
            {optional}
          </span>
        )}
      </label>
      {children}
      {hint && <p className="text-[10px] text-zinc-600 leading-relaxed">{hint}</p>}
    </div>
  );
}

function SL({ children }: { children: React.ReactNode }) {
  return <p className="text-[10px] font-bold uppercase tracking-[1.5px] text-zinc-600">{children}</p>;
}

function LogBox({ logs, t }: { logs: string[]; t: (k:string)=>string }) {
  const ref = useRef<HTMLDivElement>(null);
  useEffect(() => { ref.current?.scrollTo(0, ref.current.scrollHeight); }, [logs]);
  const cls = (l: string) => {
    if (l.includes('✅') || l.includes('Conectado') || l.includes('Connected')) return 'text-green-400';
    if (l.includes('❌') || l.includes('Erro') || l.includes('Error')) return 'text-red-400';
    if (l.includes('⚠')) return 'text-yellow-400';
    if (l.includes('▶') || l.includes('🎮') || l.includes('🖼')) return 'text-[#00a8ff]';
    return 'text-zinc-600';
  };
  return (
    <div ref={ref} className="flex-1 overflow-y-auto bg-[#0a0a0f] border border-[#2a2a3d]
      rounded-md p-2 min-h-[80px] font-mono text-[11px] leading-relaxed">
      {logs.length === 0
        ? <span className="text-zinc-700">{t('logWaiting')}</span>
        : logs.map((l, i) => <div key={i} className={cls(l)}>{l}</div>)}
    </div>
  );
}

// ── Main App ───────────────────────────────────────────────────────────

export default function App() {
  const { config, appState, logs, running, saving, saveError, saveConfig, connect, disconnect } = useTauri();

  const [form,        setForm]        = useState<Config>(DEFAULT_CONFIG);
  const [coverOpen,   setCoverOpen]   = useState(false);
  const [saveFlash,   setSaveFlash]   = useState(false);
  const [ipError,     setIpError]     = useState(false);
  const [clientError, setClientError] = useState(false);

  const lang = form.language ?? 'pt';
  const t    = useT(lang);

  useEffect(() => { setForm(config); }, [config]);
  useEffect(() => {
    if (config.igdbClientId || config.igdbClientSecret || config.customIconUrl) setCoverOpen(true);
  }, [config.igdbClientId, config.igdbClientSecret, config.customIconUrl]);

  const set = (k: keyof Config, v: string | boolean | number) =>
    setForm(f => ({ ...f, [k]: v }));

  function changeLang(l: Lang) {
    const updated = { ...form, language: l };
    setForm(updated);
    saveConfig(updated); // save language preference immediately
  }

  const hasIgdb   = !!(form.igdbClientId && form.igdbClientSecret);
  const hasCustom = !!form.customIconUrl;
  const coverBadge   = hasCustom ? t('badgeCustom') : hasIgdb ? t('badgeIgdb') : t('badgeOnly');
  const coverBadgeCls = hasCustom
    ? 'text-purple-400 border-purple-400/30 bg-purple-400/10'
    : hasIgdb
      ? 'text-green-400 border-green-400/30 bg-green-400/10'
      : 'text-zinc-600 border-[#2a2a3d]';

  async function handleSave() {
    setIpError(false); setClientError(false);
    const ok = await saveConfig(form);
    if (!ok) {
      if (saveError?.includes('IP') || saveError?.includes('MAC')) setIpError(true);
      if (saveError?.includes('Client')) setClientError(true);
      return;
    }
    setSaveFlash(true);
    setTimeout(() => setSaveFlash(false), 1500);
  }

  async function handleConnect() {
    if (running) { await disconnect(); return; }
    setIpError(false); setClientError(false);
    await connect(form);
    if (saveError?.includes('IP') || saveError?.includes('MAC')) setIpError(true);
    if (saveError?.includes('Client')) setClientError(true);
  }

  return (
    <div className="flex flex-col h-screen bg-[#0a0a0f] text-zinc-200 overflow-hidden">

      {/* Titlebar */}
      <div data-tauri-drag-region
        className="flex items-center justify-between px-4 pt-3 pb-2 bg-[#13131c] border-b border-[#2a2a3d] flex-shrink-0 select-none">
        <div className="flex items-baseline gap-2" data-tauri-drag-region>
          <span className="text-base font-black bg-gradient-to-r from-[#0050c8] to-[#00a8ff] bg-clip-text text-transparent">
            PSVita
          </span>
          <span className="text-[10px] uppercase tracking-[3px] text-zinc-600">{t('appSubtitle')}</span>
        </div>
        {/* Language switch — no drag region so clicks work */}
        <div onClick={e => e.stopPropagation()}>
          <LangSwitch lang={lang} onChange={changeLang}/>
        </div>
      </div>

      {/* Status */}
      <div className={`flex items-center gap-3 px-4 py-3 border-b flex-shrink-0 transition-all ${
        appState.status === 'connected'
          ? 'bg-gradient-to-r from-[#00a8ff]/5 to-transparent border-[#00a8ff]/20'
          : 'bg-[#13131c] border-[#2a2a3d]'
      }`}>
        <StatusDot status={appState.status}/>
        <div className="flex-1 min-w-0">
          <div className="text-sm font-semibold truncate">
            {appState.currentTitle || t(`status${appState.status.charAt(0).toUpperCase() + appState.status.slice(1)}`) || appState.statusMessage}
          </div>
          <div className="text-xs text-zinc-500 mt-px truncate">
            {appState.currentTitle
              ? (appState.coverSource ? `${t('coverVia')} ${appState.coverSource}` : t('statusConnected'))
              : (appState.discordConnected ? t('discordConnected') : '—')}
          </div>
        </div>
        <PlatformBadge platform={appState.platform} t={t}/>
      </div>

      {/* Scrollable content */}
      <div className="flex-1 overflow-y-auto">

        {/* Config */}
        <section className="px-4 py-4 border-b border-[#2a2a3d] space-y-3">
          <SL>{t('secConfig')}</SL>

          <Field label={t('fieldIp')}>
            <Input value={form.ip} onChange={e => set('ip', e.target.value)}
              placeholder="192.168.1.42 ou aa:bb:cc:dd:ee:ff"
              error={ipError} spellCheck={false} autoComplete="off"/>
          </Field>

          <Field label={t('fieldClientId')}>
            <Input value={form.clientId} onChange={e => set('clientId', e.target.value)}
              placeholder="123456789012345678" error={clientError} spellCheck={false} autoComplete="off"/>
            {form.clientId && (
              <a href={`https://discord.com/developers/applications/${form.clientId}`}
                target="_blank" rel="noreferrer"
                className="text-[11px] text-zinc-600 hover:text-[#00a8ff] transition-colors self-end">
                {t('openPortal')}
              </a>
            )}
          </Field>

          <div className="grid grid-cols-2 gap-3">
            <Field label={t('fieldInterval')}>
              <Input type="number" min={1} max={60} value={form.updateInterval}
                onChange={e => set('updateInterval', Math.max(1, parseInt(e.target.value)||10))}/>
            </Field>
            <Field label={t('fieldState')}>
              <Input value={form.state} onChange={e => set('state', e.target.value)}
                placeholder={t('fieldStatePh')}/>
            </Field>
          </div>

          <div className="grid grid-cols-2 gap-x-4 gap-y-3 pt-1">
            <Toggle id="displayTimer"    checked={form.displayTimer}    onChange={v => set('displayTimer',v)}    label={t('togTimer')}/>
            <Toggle id="displayMainMenu" checked={form.displayMainMenu} onChange={v => set('displayMainMenu',v)} label={t('togLiveArea')}/>
            <Toggle id="minimizeToTray"  checked={form.minimizeToTray}  onChange={v => set('minimizeToTray',v)}  label={t('togTray')}/>
          </div>
        </section>

        {/* Cover art */}
        <section className="px-4 py-4 border-b border-[#2a2a3d]">
          <button onClick={() => setCoverOpen(o => !o)} className="flex items-center w-full gap-2 group">
            <SL>{t('secCover')}</SL>
            <span className={`ml-1 text-[10px] font-semibold px-2 py-0.5 rounded-full border transition-all ${coverBadgeCls}`}>
              {coverBadge}
            </span>
            <span className={`ml-auto text-zinc-600 text-base transition-transform duration-200 ${coverOpen?'rotate-90':''}`}>›</span>
          </button>

          {coverOpen && (
            <div className="mt-3 space-y-3">
              <Field label={t('fieldCustomIcon')} optional={t('optional')} hint={t('fieldCustomIconHint')}>
                <Input value={form.customIconUrl} onChange={e => set('customIconUrl', e.target.value)}
                  placeholder={t('fieldCustomIconPh')} spellCheck={false}/>
              </Field>

              <div className="border-t border-[#2a2a3d] pt-3">
                <div className="flex gap-2.5 p-3 bg-[#13131c] border border-[#2a2a3d] rounded-lg mb-3">
                  <span className="text-lg">🎮</span>
                  <p className="text-xs text-zinc-500 leading-relaxed">
                    {t('coverInfo')}{' '}
                    <a href="https://dev.twitch.tv/console" target="_blank" rel="noreferrer"
                      className="text-[#00a8ff] hover:underline">{t('igdbPortal')}</a>
                  </p>
                </div>
                <div className="space-y-3">
                  <Field label={t('fieldIgdbId')} optional={t('optional')}>
                    <Input value={form.igdbClientId} onChange={e => set('igdbClientId', e.target.value)}
                      placeholder={t('fieldIgdbIdPh')} spellCheck={false} autoComplete="off"/>
                  </Field>
                  <Field label={t('fieldIgdbSec')} optional={t('optional')}>
                    <Input type="password" value={form.igdbClientSecret}
                      onChange={e => set('igdbClientSecret', e.target.value)}
                      placeholder={t('fieldIgdbSecPh')} spellCheck={false} autoComplete="off"/>
                  </Field>
                </div>
              </div>

              <div className="flex gap-2 p-2.5 bg-[#00a8ff]/5 border border-[#00a8ff]/15 rounded-md">
                <span>💡</span>
                <p className="text-[11px] text-zinc-500 leading-relaxed">{t('coverFallback')}</p>
              </div>
            </div>
          )}
        </section>

        {/* Buttons */}
        <div className="flex gap-2.5 px-4 py-3 border-b border-[#2a2a3d]">
          <button onClick={handleSave} disabled={saving}
            className="flex-1 py-2 rounded-md text-sm font-semibold bg-[#13131c] border border-[#2a2a3d]
              text-zinc-400 hover:bg-[#2a2a3d] hover:text-zinc-200 transition-all disabled:opacity-50">
            {saveFlash ? t('btnSaved') : saving ? t('btnSaving') : t('btnSave')}
          </button>
          <button onClick={handleConnect}
            className={`flex-1 py-2 rounded-md text-sm font-semibold transition-all hover:-translate-y-px active:translate-y-0 ${
              running
                ? 'bg-gradient-to-r from-red-700 to-red-500 text-white shadow-[0_2px_12px_rgba(239,68,68,0.2)]'
                : 'bg-gradient-to-r from-[#0050c8] to-[#00a8ff] text-white shadow-[0_2px_12px_rgba(0,168,255,0.2)]'
            }`}>
            {running ? t('btnDisconnect') : t('btnConnect')}
          </button>
        </div>

        {saveError && !saveFlash && (
          <p className="mx-4 mt-2 text-xs text-red-400">{saveError}</p>
        )}

        {/* Log */}
        <section className="flex flex-col px-4 py-3 gap-2 min-h-[100px]">
          <SL>{t('secLog')}</SL>
          <LogBox logs={logs} t={t}/>
        </section>
      </div>
    </div>
  );
}
