export interface Config {
  ip:                string;
  clientId:          string;
  updateInterval:    number;
  state:             string;
  displayTimer:      boolean;
  displayMainMenu:   boolean;
  minimizeToTray:    boolean;
  customIconUrl:     string;
  igdbClientId:      string;
  igdbClientSecret:  string;
  language:          'pt' | 'en';
}

export const DEFAULT_CONFIG: Config = {
  ip:               '',
  clientId:         '',
  updateInterval:   10,
  state:            '',
  displayTimer:     true,
  displayMainMenu:  true,
  minimizeToTray:   true,
  customIconUrl:    '',
  igdbClientId:     '',
  igdbClientSecret: '',
  language:         'pt',
};

export interface AppState {
  status:           'disconnected' | 'connecting' | 'connected' | 'error';
  currentTitle:     string;
  currentTitleId:   string;
  platform:         string;
  discordConnected: boolean;
  statusMessage:    string;
  coverUrl:         string | null;
  coverSource:      string;
}

export const DEFAULT_STATE: AppState = {
  status:           'disconnected',
  currentTitle:     '',
  currentTitleId:   '',
  platform:         '',
  discordConnected: false,
  statusMessage:    'Desconectado',
  coverUrl:         null,
  coverSource:      '',
};

// ── i18n ───────────────────────────────────────────────────────────────

export type Lang = 'pt' | 'en';

export const T: Record<Lang, Record<string, string>> = {
  pt: {
    // titlebar
    appSubtitle:       'PRESENCE',
    // status
    statusDisconnected:'Desconectado',
    statusConnecting:  'Conectando...',
    statusConnected:   'Conectado',
    discordConnected:  'Discord conectado',
    coverVia:          'Capa via',
    // sections
    secConfig:         'Configuração',
    secCover:          'Capa de Jogos',
    secLog:            'Log',
    // fields
    fieldIp:           'IP ou MAC do PS Vita',
    fieldClientId:     'Discord Client ID',
    fieldInterval:     'Intervalo (segundos)',
    fieldState:        'Estado customizado',
    fieldStatePh:      'Jogando no Vita',
    fieldCustomIcon:   'Ícone customizado',
    fieldCustomIconPh: 'https://exemplo.com/icone.png',
    fieldCustomIconHint:'URL pública de imagem PNG/JPG. Tem prioridade sobre IGDB e GitHub.',
    fieldIgdbId:       'Twitch / IGDB Client ID',
    fieldIgdbSec:      'Twitch / IGDB Client Secret',
    fieldIgdbIdPh:     'Cole seu Twitch Client ID',
    fieldIgdbSecPh:    'Cole seu Twitch Client Secret',
    // toggles
    togTimer:          'Mostrar timer',
    togLiveArea:       'Mostrar LiveArea',
    togTray:           'Minimizar pro tray',
    // buttons
    btnSave:           'Salvar',
    btnSaved:          '✓ Salvo',
    btnSaving:         'Salvando...',
    btnConnect:        'Conectar',
    btnDisconnect:     'Desconectar',
    // badges
    badgeOnly:         'Apenas GitHub',
    badgeIgdb:         'IGDB + GitHub',
    badgeCustom:       'Ícone custom',
    badgePsVita:       'PS VITA',
    badgePsp:          'PSP',
    badgeEmulator:     'EMULADOR',
    // optional label
    optional:          'opcional',
    // portal links
    openPortal:        'Abrir Developer Portal ↗',
    igdbPortal:        'dev.twitch.tv ↗',
    // cover info
    coverInfo:         'Busca capas automaticamente no IGDB. Requer conta gratuita em',
    coverFallback:     'Sem credenciais IGDB, capas são buscadas por Title ID no GitHub (PSMT-Covers + NutDB).',
    // log
    logWaiting:        'Aguardando...',
    // platforms
    platVita:          'PS VITA',
    platPsp:           'PSP',
    platEmulator:      'EMULADOR',
  },
  en: {
    appSubtitle:       'PRESENCE',
    statusDisconnected:'Disconnected',
    statusConnecting:  'Connecting...',
    statusConnected:   'Connected',
    discordConnected:  'Discord connected',
    coverVia:          'Cover via',
    secConfig:         'Configuration',
    secCover:          'Game Covers',
    secLog:            'Log',
    fieldIp:           'PS Vita IP or MAC',
    fieldClientId:     'Discord Client ID',
    fieldInterval:     'Interval (seconds)',
    fieldState:        'Custom state',
    fieldStatePh:      'Playing on Vita',
    fieldCustomIcon:   'Custom icon',
    fieldCustomIconPh: 'https://example.com/icon.png',
    fieldCustomIconHint:'Public PNG/JPG image URL. Takes priority over IGDB and GitHub.',
    fieldIgdbId:       'Twitch / IGDB Client ID',
    fieldIgdbSec:      'Twitch / IGDB Client Secret',
    fieldIgdbIdPh:     'Paste your Twitch Client ID',
    fieldIgdbSecPh:    'Paste your Twitch Client Secret',
    togTimer:          'Show timer',
    togLiveArea:       'Show LiveArea',
    togTray:           'Minimize to tray',
    btnSave:           'Save',
    btnSaved:          '✓ Saved',
    btnSaving:         'Saving...',
    btnConnect:        'Connect',
    btnDisconnect:     'Disconnect',
    badgeOnly:         'GitHub only',
    badgeIgdb:         'IGDB + GitHub',
    badgeCustom:       'Custom icon',
    badgePsVita:       'PS VITA',
    badgePsp:          'PSP',
    badgeEmulator:     'EMULATOR',
    optional:          'optional',
    openPortal:        'Open Developer Portal ↗',
    igdbPortal:        'dev.twitch.tv ↗',
    coverInfo:         'Automatically fetches game covers from IGDB. Requires a free account at',
    coverFallback:     'Without IGDB credentials, covers are fetched by Title ID from GitHub (PSMT-Covers + NutDB).',
    logWaiting:        'Waiting...',
    platVita:          'PS VITA',
    platPsp:           'PSP',
    platEmulator:      'EMULATOR',
  },
};
