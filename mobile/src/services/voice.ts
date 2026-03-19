/**
 * VoiceService — voice interaction pipeline for RuVix mobile app.
 *
 * In demo mode (default), simulates the full voice pipeline locally.
 * When the RLMX server is reachable, delegates intent decomposition
 * to the server's MCP endpoint.
 *
 * Also exports functional stubs (initVoice, startListening, stopListening,
 * cleanup) for compatibility with components that use the simpler API.
 */

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export type VoicePersona =
  | 'Finance'
  | 'Health'
  | 'Legal'
  | 'Shopping'
  | 'Calendar'
  | 'Emergency';

export interface AgentProgressUpdate {
  domain: VoicePersona;
  progress: number; // 0-100
  status: 'queued' | 'working' | 'done' | 'error';
  detail?: string;
}

export interface VoiceResponse {
  persona: VoicePersona;
  text: string;
  confidence: number;
  timestamp: number;
}

export interface IntentResult {
  intents: Array<{
    persona: VoicePersona;
    action: string;
    entities: Record<string, string>;
  }>;
  raw: string;
}

export interface VoiceInteractionResult {
  transcript: string;
  intents: IntentResult;
  progress: AgentProgressUpdate[];
  responses: VoiceResponse[];
}

export interface VoiceCallbacks {
  onTranscript: (text: string) => void;
  onListeningChange: (listening: boolean) => void;
  onError: (error: string) => void;
}

// ---------------------------------------------------------------------------
// Persona keyword map for intent parsing
// ---------------------------------------------------------------------------

const PERSONA_KEYWORDS: Record<VoicePersona, string[]> = {
  Finance: [
    'money', 'bank', 'savings', 'budget', 'invest', 'payment', 'bill',
    'expense', 'income', 'stock', 'crypto', 'transfer', 'account', 'loan',
    'credit', 'debit', 'salary', 'tax', 'finance', 'spend', 'save', 'dollar',
    'spending', 'subscriptions',
  ],
  Health: [
    'health', 'doctor', 'appointment', 'medicine', 'exercise', 'gym',
    'sleep', 'diet', 'calories', 'steps', 'heart', 'blood', 'pressure',
    'weight', 'wellness', 'symptom', 'prescription', 'vaccine', 'therapy',
    'walk',
  ],
  Legal: [
    'legal', 'contract', 'lawyer', 'court', 'lawsuit', 'agreement',
    'terms', 'policy', 'compliance', 'regulation', 'license', 'patent',
    'trademark', 'dispute', 'arbitration', 'liability', 'passwords',
    'compromised',
  ],
  Shopping: [
    'buy', 'shop', 'order', 'price', 'deal', 'discount', 'coupon',
    'cart', 'delivery', 'return', 'product', 'compare', 'review',
    'amazon', 'store', 'purchase', 'gift', 'sale',
  ],
  Calendar: [
    'schedule', 'meeting', 'calendar', 'remind', 'reminder', 'event',
    'tomorrow', 'today', 'week', 'appointment', 'deadline', 'plan',
    'morning', 'afternoon', 'evening', 'date', 'time', 'alarm',
    'meetings', 'deep work', 'optimize',
  ],
  Emergency: [
    'emergency', 'urgent', 'help', 'danger', 'fire', 'police',
    'ambulance', '911', 'accident', 'alert', 'warning', 'critical',
    'sos', 'panic',
  ],
};

// ---------------------------------------------------------------------------
// Simulated response templates
// ---------------------------------------------------------------------------

const DEMO_RESPONSES: Record<VoicePersona, string[]> = {
  Finance: [
    'I found a potential savings of $47.30 by switching your streaming subscriptions.',
    'Your monthly spending is tracking 12% under budget. Great progress!',
    'I noticed a recurring charge of $9.99 you may want to review.',
  ],
  Health: [
    'You have a check-up scheduled for next Tuesday at 10 AM.',
    'Your step count is averaging 8,200 steps this week, up 15% from last week.',
    'Based on your sleep data, try going to bed 30 minutes earlier tonight.',
  ],
  Legal: [
    'I reviewed the contract and flagged 3 clauses that need attention.',
    'Your NDA expires in 14 days. Would you like to renew?',
    'The terms of service update contains a new arbitration clause.',
  ],
  Shopping: [
    'I found the item 23% cheaper on a different retailer.',
    'Your package is out for delivery and should arrive by 5 PM.',
    'The price dropped on 2 items in your watchlist.',
  ],
  Calendar: [
    'You have 3 meetings tomorrow. The first is at 9 AM with the design team.',
    'I have set a reminder for your dentist appointment on Friday.',
    'Your week looks open on Thursday afternoon if you need focus time.',
  ],
  Emergency: [
    'Emergency contacts have been notified. Stay on the line.',
    'I am locating the nearest hospital, 0.8 miles away.',
    'Alert sent to your emergency contacts with your current location.',
  ],
};

// ---------------------------------------------------------------------------
// VoiceService (class API for hooks)
// ---------------------------------------------------------------------------

export class VoiceService {
  private isRecording = false;
  private serverUrl: string;
  private demoMode = true;

  constructor(serverUrl = 'http://localhost:3000') {
    this.serverUrl = serverUrl;
    this.checkServerAvailability();
  }

  private async checkServerAvailability(): Promise<void> {
    try {
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), 2000);
      const res = await fetch(`${this.serverUrl}/health`, {
        signal: controller.signal,
      });
      clearTimeout(timeoutId);
      this.demoMode = !res.ok;
    } catch {
      this.demoMode = true;
    }
  }

  isDemoMode(): boolean {
    return this.demoMode;
  }

  async startListening(): Promise<void> {
    if (this.isRecording) {return;}
    this.isRecording = true;
  }

  async stopListening(): Promise<string> {
    if (!this.isRecording) {return '';}
    this.isRecording = false;
    return '';
  }

  getIsRecording(): boolean {
    return this.isRecording;
  }

  async processTranscript(text: string): Promise<IntentResult> {
    if (!this.demoMode) {
      try {
        const res = await fetch(`${this.serverUrl}/rpc`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            jsonrpc: '2.0',
            id: Date.now(),
            method: 'rlmx_voice_intent',
            params: { transcript: text },
          }),
        });
        const data = await res.json();
        if (data.result) {return data.result as IntentResult;}
      } catch {
        // Fall through to local parsing
      }
    }

    return this.parseIntentsLocally(text);
  }

  private parseIntentsLocally(text: string): IntentResult {
    const lower = text.toLowerCase();
    const matched: IntentResult['intents'] = [];

    for (const [persona, keywords] of Object.entries(PERSONA_KEYWORDS)) {
      const hits = keywords.filter((kw) => lower.includes(kw));
      if (hits.length > 0) {
        matched.push({
          persona: persona as VoicePersona,
          action: `handle_${persona.toLowerCase()}`,
          entities: { keywords: hits.join(', ') },
        });
      }
    }

    if (matched.length === 0) {
      matched.push({
        persona: 'Calendar',
        action: 'handle_general',
        entities: { raw: text },
      });
    }

    return { intents: matched, raw: text };
  }

  async simulateVoiceInteraction(
    text: string,
    onProgress?: (update: AgentProgressUpdate) => void,
  ): Promise<VoiceInteractionResult> {
    const intents = await this.processTranscript(text);
    const progress: AgentProgressUpdate[] = [];
    const responses: VoiceResponse[] = [];

    for (const intent of intents.intents) {
      const queued: AgentProgressUpdate = {
        domain: intent.persona,
        progress: 0,
        status: 'queued',
        detail: `Processing ${intent.persona} request...`,
      };
      progress.push(queued);
      onProgress?.(queued);

      await delay(300 + Math.random() * 400);
      const working: AgentProgressUpdate = {
        domain: intent.persona,
        progress: 50,
        status: 'working',
        detail: `Agent analyzing ${intent.action}...`,
      };
      progress.push(working);
      onProgress?.(working);

      await delay(400 + Math.random() * 600);
      const done: AgentProgressUpdate = {
        domain: intent.persona,
        progress: 100,
        status: 'done',
        detail: 'Complete',
      };
      progress.push(done);
      onProgress?.(done);

      const templates = DEMO_RESPONSES[intent.persona];
      const responseText =
        templates[Math.floor(Math.random() * templates.length)];
      responses.push({
        persona: intent.persona,
        text: responseText,
        confidence: 0.85 + Math.random() * 0.15,
        timestamp: Date.now(),
      });
    }

    return { transcript: text, intents, progress, responses };
  }
}

// ---------------------------------------------------------------------------
// Functional API (for compatibility with simpler components)
// ---------------------------------------------------------------------------

let _callbacks: VoiceCallbacks | null = null;
let _simulationTimeout: ReturnType<typeof setTimeout> | null = null;

export function initVoice(cb: VoiceCallbacks): void {
  _callbacks = cb;
}

export function startListening(): void {
  if (!_callbacks) {return;}
  _callbacks.onListeningChange(true);

  const phrases = [
    'Check my spending this week and find savings',
    'What meetings do I have tomorrow?',
    'How many steps did I walk today?',
    'Are any of my passwords compromised?',
    'Optimize my schedule for deep work',
    'Cancel unused subscriptions',
  ];

  const phrase = phrases[Math.floor(Math.random() * phrases.length)];
  let index = 0;

  const typeInterval = setInterval(() => {
    index += Math.floor(Math.random() * 3) + 2;
    if (index >= phrase.length) {
      index = phrase.length;
      clearInterval(typeInterval);
      _simulationTimeout = setTimeout(() => {
        _callbacks?.onListeningChange(false);
      }, 500);
    }
    _callbacks?.onTranscript(phrase.slice(0, index));
  }, 80);

  _simulationTimeout = setTimeout(() => {
    clearInterval(typeInterval);
  }, 5000);
}

export function stopListening(): void {
  if (_simulationTimeout) {
    clearTimeout(_simulationTimeout);
    _simulationTimeout = null;
  }
  _callbacks?.onListeningChange(false);
}

export function cleanup(): void {
  stopListening();
  _callbacks = null;
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function delay(ms: number): Promise<void> {
  return new Promise<void>((resolve) => setTimeout(resolve, ms));
}

// Singleton
let _instance: VoiceService | null = null;

export function getVoiceService(serverUrl?: string): VoiceService {
  if (!_instance) {
    _instance = new VoiceService(serverUrl);
  }
  return _instance;
}
