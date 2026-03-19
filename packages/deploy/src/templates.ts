import { LifeDomain, AgentType, SyscallPermission } from '@aix/shared';
import type { AgentManifest, Modality } from './manifest.js';

function template(
  id: string,
  name: string,
  lifeDomain: LifeDomain,
  domainTags: string[],
  overrides: Partial<AgentManifest> = {},
): AgentManifest {
  return {
    id,
    name,
    version: '0.1.0',
    origin: overrides.origin ?? { type: 'rlmx', agentType: AgentType.Worker, permissions: [SyscallPermission.VecSearch] },
    lifeDomain,
    domainTags,
    capabilities: overrides.capabilities ?? [{ name: 'process', required: true }],
    modalities: overrides.modalities ?? ['text'],
    resourceEnvelope: overrides.resourceEnvelope ?? { cpuCores: 2, memoryMb: 512, diskMb: 1024, maxRuntimeMs: 300000 },
    transports: overrides.transports ?? [{ type: 'mcp', endpoint: 'http://localhost:3000' }],
    discovery: overrides.discovery,
    security: overrides.security ?? { authMethod: 'bearer' },
    deployment: overrides.deployment ?? { profiles: ['rlmx-edge'] },
    sensorConfig: overrides.sensorConfig,
    learning: overrides.learning,
    metadata: overrides.metadata ?? {},
  };
}

function seedSensorTemplate(
  id: string,
  name: string,
  domainTags: string[],
  sensorInterfaces: ('gpio' | 'i2c' | 'spi' | 'uart' | 'usb')[] = ['gpio', 'i2c'],
): AgentManifest {
  return template(id, name, LifeDomain.Home, domainTags, {
    origin: { type: 'seed', mcpEndpoint: 'http://localhost:5353/mcp', sensorProfile: id },
    modalities: ['sensor'],
    resourceEnvelope: { cpuCores: 1, memoryMb: 256, diskMb: 512, maxRuntimeMs: 0 },
    transports: [
      { type: 'mcp', endpoint: 'http://localhost:5353/mcp' },
      { type: 'rest', baseUrl: 'https://localhost:8443' },
    ],
    discovery: { serviceType: '_cognitum._tcp', port: 5353 },
    security: { authMethod: 'bearer', attestation: 'ed25519' },
    deployment: { profiles: ['seed-compatible'] },
    sensorConfig: { interfaces: sensorInterfaces, driftDetection: true, samplingIntervalMs: 1000 },
    learning: { sona: false, federation: true, driftDetectors: ['ewma'] },
  });
}

// === Finance (6) ===
const billNegotiator = template('bill-negotiator', 'Bill Negotiator', LifeDomain.Finance, ['budgeting', 'expense'], {
  capabilities: [{ name: 'negotiate', required: true }, { name: 'search', required: true }],
  learning: { sona: true, federation: true },
});
const taxPlanner = template('tax-planner', 'Tax Planner', LifeDomain.Finance, ['tax', 'accounting'], {
  origin: { type: 'rlmx', agentType: AgentType.Analyst, permissions: [SyscallPermission.VecSearch, SyscallPermission.GraphQuery] },
});
const investmentTracker = template('investment-tracker', 'Investment Tracker', LifeDomain.Finance, ['investment', 'stocks', 'trading']);
const expenseAnalyzer = template('expense-analyzer', 'Expense Analyzer', LifeDomain.Finance, ['expense', 'budgeting']);
const budgetOptimizer = template('budget-optimizer', 'Budget Optimizer', LifeDomain.Finance, ['budgeting', 'accounting'], {
  learning: { sona: true, federation: true },
});
const cryptoMonitor = template('crypto-monitor', 'Crypto Monitor', LifeDomain.Finance, ['crypto', 'trading'], {
  modalities: ['text', 'multimodal'],
});

// === Health (6) ===
const medicationReminder = template('medication-reminder', 'Medication Reminder', LifeDomain.Health, ['medication', 'medical'], {
  modalities: ['text', 'voice'],
  origin: { type: 'rlmx', agentType: AgentType.Worker, permissions: [SyscallPermission.VoiceSynthesize, SyscallPermission.VecSearch] },
});
const fitnessTracker = template('fitness-tracker', 'Fitness Tracker', LifeDomain.Health, ['fitness', 'nutrition']);
const nutritionAdvisor = template('nutrition-advisor', 'Nutrition Advisor', LifeDomain.Health, ['nutrition', 'medical']);
const sleepOptimizer = template('sleep-optimizer', 'Sleep Optimizer', LifeDomain.Health, ['sleep', 'mental-wellness'], {
  learning: { sona: true, federation: true },
});
const symptomChecker = template('symptom-checker', 'Symptom Checker', LifeDomain.Health, ['symptom', 'medical'], {
  capabilities: [{ name: 'diagnose', required: true }, { name: 'search', required: true }],
});
const mentalWellness = template('mental-wellness', 'Mental Wellness', LifeDomain.Health, ['mental-wellness', 'therapy'], {
  modalities: ['text', 'voice'],
});

// === Legal (4) ===
const contractReviewer = template('contract-reviewer', 'Contract Reviewer', LifeDomain.Legal, ['contract', 'compliance']);
const rightsAdvisor = template('rights-advisor', 'Rights Advisor', LifeDomain.Legal, ['rights', 'regulation']);
const complianceChecker = template('compliance-checker', 'Compliance Checker', LifeDomain.Legal, ['compliance', 'regulation']);
const disputeResolver = template('dispute-resolver', 'Dispute Resolver', LifeDomain.Legal, ['dispute', 'contract']);

// === Home — IoT/Sensor (6, origin: seed) ===
const temperatureMonitor = seedSensorTemplate('temperature-monitor', 'Temperature Monitor', ['temperature', 'sensor', 'iot']);
const humidityTracker = seedSensorTemplate('humidity-tracker', 'Humidity Tracker', ['humidity', 'sensor', 'iot']);
const motionDetector = seedSensorTemplate('motion-detector', 'Motion Detector', ['motion', 'sensor', 'security-cam']);
const lightController = seedSensorTemplate('light-controller', 'Light Controller', ['light', 'lighting', 'iot'], ['gpio']);
const energyMeter = seedSensorTemplate('energy-meter', 'Energy Meter', ['energy-meter', 'iot'], ['i2c', 'spi']);
const waterLeakDetector = seedSensorTemplate('water-leak-detector', 'Water Leak Detector', ['water-leak', 'sensor', 'iot']);

// === Home — Automation (6, origin: rlmx) ===
const thermostatAgent = template('thermostat-agent', 'Thermostat Agent', LifeDomain.Home, ['thermostat', 'temperature'], {
  modalities: ['text', 'voice'],
});
const lightingAgent = template('lighting-agent', 'Lighting Agent', LifeDomain.Home, ['lighting', 'light']);
const securityCamAgent = template('security-cam-agent', 'Security Cam Agent', LifeDomain.Home, ['security-cam', 'motion'], {
  modalities: ['text', 'vision'],
});
const applianceScheduler = template('appliance-scheduler', 'Appliance Scheduler', LifeDomain.Home, ['appliance', 'iot']);
const doorbellAgent = template('doorbell-agent', 'Doorbell Agent', LifeDomain.Home, ['doorbell', 'motion'], {
  modalities: ['text', 'voice', 'vision'],
});
const garageAgent = template('garage-agent', 'Garage Agent', LifeDomain.Home, ['garage', 'iot']);

// === Career — Industrial (4, hybrid seed+rlmx) ===
const predictiveMaintenance = template('predictive-maintenance', 'Predictive Maintenance', LifeDomain.Career, ['predictive-maintenance', 'industrial', 'vibration'], {
  origin: { type: 'rlmx', agentType: AgentType.Analyst, permissions: [SyscallPermission.VecSearch, SyscallPermission.GraphQuery] },
  modalities: ['text', 'sensor'],
  deployment: { profiles: ['hybrid'] },
  learning: { sona: true, federation: true, driftDetectors: ['ewma', 'page-hinkley'] },
});
const qualityInspector = template('quality-inspector', 'Quality Inspector', LifeDomain.Career, ['quality', 'manufacturing'], {
  modalities: ['text', 'vision'],
  deployment: { profiles: ['hybrid'] },
});
const energyOptimizer = template('energy-optimizer', 'Energy Optimizer', LifeDomain.Career, ['energy-meter', 'industrial'], {
  deployment: { profiles: ['hybrid'] },
});
const vibrationAnalyzer = template('vibration-analyzer', 'Vibration Analyzer', LifeDomain.Career, ['vibration', 'predictive-maintenance'], {
  modalities: ['text', 'sensor'],
  deployment: { profiles: ['hybrid'] },
});

// === Home — Agricultural (4, origin: seed) ===
const soilMonitor = seedSensorTemplate('soil-monitor', 'Soil Monitor', ['soil', 'agriculture', 'farming'], ['i2c', 'spi']);
const irrigationController = seedSensorTemplate('irrigation-controller', 'Irrigation Controller', ['irrigation', 'agriculture'], ['gpio']);
const cropHealthAnalyzer = seedSensorTemplate('crop-health-analyzer', 'Crop Health Analyzer', ['crops', 'agriculture', 'farming']);
const greenhouseManager = seedSensorTemplate('greenhouse-manager', 'Greenhouse Manager', ['greenhouse', 'temperature', 'humidity']);

// === Home — Environmental (4, origin: seed) ===
const airQualityMonitor = seedSensorTemplate('air-quality-monitor', 'Air Quality Monitor', ['air-quality', 'environmental'], ['i2c']);
const waterQualityMonitor = seedSensorTemplate('water-quality-monitor', 'Water Quality Monitor', ['water-quality', 'environmental'], ['i2c', 'uart']);
const weatherForecaster = seedSensorTemplate('weather-forecaster', 'Weather Forecaster', ['weather', 'environmental'], ['i2c', 'spi']);
const uvMonitor = seedSensorTemplate('uv-monitor', 'UV Monitor', ['uv', 'environmental'], ['i2c']);

// === Education (4) ===
const studyPlanner = template('study-planner', 'Study Planner', LifeDomain.Education, ['study', 'course']);
const flashcardTutor = template('flashcard-tutor', 'Flashcard Tutor', LifeDomain.Education, ['flashcard', 'study'], {
  modalities: ['text', 'voice'],
});
const languageCoach = template('language-coach', 'Language Coach', LifeDomain.Education, ['language', 'tutoring'], {
  modalities: ['text', 'voice'],
  learning: { sona: true, federation: true },
});
const skillTracker = template('skill-tracker', 'Skill Tracker', LifeDomain.Education, ['certification', 'course']);

// === Travel (3) ===
const flightTracker = template('flight-tracker', 'Flight Tracker', LifeDomain.Travel, ['flight', 'itinerary']);
const itineraryBuilder = template('itinerary-builder', 'Itinerary Builder', LifeDomain.Travel, ['itinerary', 'hotel', 'flight']);
const currencyConverter = template('currency-converter', 'Currency Converter', LifeDomain.Travel, ['currency', 'travel']);

// === Social (3) ===
const eventPlanner = template('event-planner', 'Event Planner', LifeDomain.Social, ['event', 'party']);
const birthdayReminder = template('birthday-reminder', 'Birthday Reminder', LifeDomain.Social, ['birthday', 'gift'], {
  modalities: ['text', 'voice'],
});
const giftSuggester = template('gift-suggester', 'Gift Suggester', LifeDomain.Social, ['gift', 'shopping']);

// === Pet (2) ===
const feedingScheduler = template('feeding-scheduler', 'Feeding Scheduler', LifeDomain.Pet, ['feeding', 'pet-sitting']);
const vetReminder = template('vet-reminder', 'Vet Reminder', LifeDomain.Pet, ['vet', 'health-log']);

// === Shopping (2) ===
const priceComparator = template('price-comparator', 'Price Comparator', LifeDomain.Shopping, ['price-compare', 'deal-finder']);
const dealFinder = template('deal-finder', 'Deal Finder', LifeDomain.Shopping, ['deal-finder', 'coupon']);

// === Bridge Templates (ADR-033) ===
const claudeCodeBridge = template('claude-code-bridge', 'Claude Code Bridge', LifeDomain.Career, ['llm', 'bridge', 'claude'], {
  origin: { type: 'external', protocol: 'bridge:claude-code', endpoint: 'cli://claude' },
  transports: [{ type: 'bridge' as const, runtime: 'claude-code' as const, config: { command: 'claude' } }],
  capabilities: [{ name: 'inference', required: true }, { name: 'code-generation', required: true }],
  modalities: ['text'],
  security: { authMethod: 'none' },
  deployment: { profiles: ['local-cli'] },
});

const codexBridge = template('codex-bridge', 'Codex Bridge', LifeDomain.Career, ['llm', 'bridge', 'codex'], {
  origin: { type: 'external', protocol: 'bridge:codex', endpoint: 'https://api.openai.com/v1/responses' },
  transports: [{ type: 'bridge' as const, runtime: 'codex' as const, config: { endpoint: 'https://api.openai.com/v1/responses', model: 'codex-mini-latest', authEnvVar: 'OPENAI_API_KEY' } }],
  capabilities: [{ name: 'inference', required: true }, { name: 'code-generation', required: true }],
  modalities: ['text'],
  security: { authMethod: 'bearer' },
  deployment: { profiles: ['cloud'] },
});

const cursorBridge = template('cursor-bridge', 'Cursor Bridge', LifeDomain.Career, ['llm', 'bridge', 'cursor'], {
  origin: { type: 'external', protocol: 'bridge:cursor', endpoint: 'ipc://cursor' },
  transports: [{ type: 'bridge' as const, runtime: 'cursor' as const, config: {} }],
  capabilities: [{ name: 'inference', required: true }, { name: 'code-editing', required: true }],
  modalities: ['text'],
  security: { authMethod: 'none' },
  deployment: { profiles: ['local-ide'] },
});

const genericLlmBridge = template('generic-llm-bridge', 'Generic LLM Bridge', LifeDomain.Career, ['llm', 'bridge', 'openai-compatible'], {
  origin: { type: 'external', protocol: 'bridge:http-generic', endpoint: 'http://localhost:8080' },
  transports: [{ type: 'bridge' as const, runtime: 'http-generic' as const, config: { endpoint: 'http://localhost:8080', model: 'default' } }],
  capabilities: [{ name: 'inference', required: true }],
  modalities: ['text'],
  security: { authMethod: 'bearer' },
  deployment: { profiles: ['local', 'cloud'] },
});

// === Template registry with pre-built indexes ===
const ALL_TEMPLATES: readonly AgentManifest[] = Object.freeze([
  // Finance
  billNegotiator, taxPlanner, investmentTracker, expenseAnalyzer, budgetOptimizer, cryptoMonitor,
  // Health
  medicationReminder, fitnessTracker, nutritionAdvisor, sleepOptimizer, symptomChecker, mentalWellness,
  // Legal
  contractReviewer, rightsAdvisor, complianceChecker, disputeResolver,
  // Home — IoT
  temperatureMonitor, humidityTracker, motionDetector, lightController, energyMeter, waterLeakDetector,
  // Home — Automation
  thermostatAgent, lightingAgent, securityCamAgent, applianceScheduler, doorbellAgent, garageAgent,
  // Career — Industrial
  predictiveMaintenance, qualityInspector, energyOptimizer, vibrationAnalyzer,
  // Home — Agricultural
  soilMonitor, irrigationController, cropHealthAnalyzer, greenhouseManager,
  // Home — Environmental
  airQualityMonitor, waterQualityMonitor, weatherForecaster, uvMonitor,
  // Education
  studyPlanner, flashcardTutor, languageCoach, skillTracker,
  // Travel
  flightTracker, itineraryBuilder, currencyConverter,
  // Social
  eventPlanner, birthdayReminder, giftSuggester,
  // Pet
  feedingScheduler, vetReminder,
  // Shopping
  priceComparator, dealFinder,
  // Bridge (ADR-033)
  claudeCodeBridge, codexBridge, cursorBridge, genericLlmBridge,
]);

// Deep-freeze all templates to prevent mutation
for (const t of ALL_TEMPLATES) Object.freeze(t);

// Pre-built indexes for O(1) lookups
const BY_ID = new Map<string, AgentManifest>(ALL_TEMPLATES.map(t => [t.id, t]));
const BY_DOMAIN = new Map<LifeDomain, AgentManifest[]>();
const BY_ORIGIN = new Map<string, AgentManifest[]>();
const BY_MODALITY = new Map<Modality, AgentManifest[]>();
for (const t of ALL_TEMPLATES) {
  const domainList = BY_DOMAIN.get(t.lifeDomain);
  if (domainList) domainList.push(t);
  else BY_DOMAIN.set(t.lifeDomain, [t]);

  const originList = BY_ORIGIN.get(t.origin.type);
  if (originList) originList.push(t);
  else BY_ORIGIN.set(t.origin.type, [t]);

  for (const mod of t.modalities) {
    const modList = BY_MODALITY.get(mod);
    if (modList) modList.push(t);
    else BY_MODALITY.set(mod, [t]);
  }
}

/** Get all 50 built-in agent templates (read-only). */
export function getTemplates(): readonly AgentManifest[] {
  return ALL_TEMPLATES;
}

/** Get a template by ID. O(1) lookup. */
export function getTemplate(id: string): AgentManifest | undefined {
  return BY_ID.get(id);
}

/** Search templates by domain. O(1) bucket lookup. */
export function getTemplatesByDomain(domain: LifeDomain): AgentManifest[] {
  return BY_DOMAIN.get(domain) ?? [];
}

/** Search templates by origin type. O(1) bucket lookup. */
export function getTemplatesByOrigin(originType: string): AgentManifest[] {
  return BY_ORIGIN.get(originType) ?? [];
}

/** Search templates by modality. O(1) bucket lookup. */
export function getTemplatesByModality(modality: Modality): AgentManifest[] {
  return BY_MODALITY.get(modality) ?? [];
}

/** Total template count. */
export const TEMPLATE_COUNT = ALL_TEMPLATES.length;
