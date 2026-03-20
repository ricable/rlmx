export type Domain =
  | 'Finance'
  | 'Health'
  | 'Legal'
  | 'Career'
  | 'Education'
  | 'Home'
  | 'Shopping'
  | 'Travel'
  | 'Social'
  | 'Government'
  | 'Automotive'
  | 'Pet';

export type PriceTier = 'Free' | 'Plus' | 'Pro';

export interface AgentReview {
  id: string;
  author: string;
  rating: number;
  text: string;
  date: string;
}

export interface MarketplaceAgent {
  id: string;
  name: string;
  publisher: string;
  domain: Domain;
  description: string;
  longDescription: string;
  rating: number;
  ratingCount: number;
  installs: number;
  price: PriceTier;
  icon: string;
  iconColor: string;
  permissions: string[];
  reviews: AgentReview[];
  featured: boolean;
  version: string;
}

export interface CollectedAgent {
  id: string;
  marketplaceId: string;
  name: string;
  domain: Domain;
  icon: string;
  iconColor: string;
  level: number;
  xp: number;
  xpToNext: number;
  unlocked: boolean;
  lastUsed?: string;
}

export const DOMAINS: Domain[] = [
  'Finance',
  'Health',
  'Legal',
  'Career',
  'Education',
  'Home',
  'Shopping',
  'Travel',
  'Social',
  'Government',
  'Automotive',
  'Pet',
];

export const DOMAIN_ICONS: Record<Domain, string> = {
  Finance: 'attach-money',
  Health: 'favorite',
  Legal: 'gavel',
  Career: 'work',
  Education: 'school',
  Home: 'home',
  Shopping: 'shopping-cart',
  Travel: 'flight',
  Social: 'people',
  Government: 'account-balance',
  Automotive: 'directions-car',
  Pet: 'pets',
};

export const DOMAIN_COLORS: Record<Domain, string> = {
  Finance: '#22c55e',
  Health: '#ec4899',
  Legal: '#f59e0b',
  Career: '#3b82f6',
  Education: '#8b5cf6',
  Home: '#06b6d4',
  Shopping: '#f97316',
  Travel: '#14b8a6',
  Social: '#a855f7',
  Government: '#64748b',
  Automotive: '#ef4444',
  Pet: '#84cc16',
};

export const LEVEL_COLORS: Record<number, string> = {
  1: '#64748b',
  2: '#94a3b8',
  3: '#22c55e',
  4: '#3b82f6',
  5: '#8b5cf6',
  6: '#a855f7',
  7: '#ec4899',
  8: '#f59e0b',
  9: '#ef4444',
  10: '#00e5ff',
};

export const LEVEL_XP_REQUIREMENTS = [
  0, 100, 300, 600, 1000, 1500, 2200, 3000, 4000, 5200,
];

export function formatInstalls(n: number): string {
  if (n >= 1_000_000) {
    return `${(n / 1_000_000).toFixed(1)}M`;
  }
  if (n >= 1_000) {
    return `${(n / 1_000).toFixed(0)}K`;
  }
  return `${n}`;
}

export const MARKETPLACE_AGENTS: MarketplaceAgent[] = [
  {
    id: 'agent-001',
    name: 'Bill Negotiator',
    publisher: 'RuVix Labs',
    domain: 'Finance',
    description: 'Automatically negotiates your bills and subscriptions for lower rates.',
    longDescription:
      'Bill Negotiator uses advanced AI to analyze your recurring bills, identify overcharges, and negotiate better rates with service providers. It handles cable, internet, phone, insurance, and subscription services. On average, users save $840 per year.\n\nThe agent monitors your bills monthly and proactively contacts providers when rates increase or when better deals become available. It keeps a full audit trail of all negotiations.',
    rating: 4.8,
    ratingCount: 24500,
    installs: 1_200_000,
    price: 'Free',
    icon: 'receipt-long',
    iconColor: '#22c55e',
    permissions: ['Read financial accounts', 'Send messages on your behalf', 'Access contact information'],
    reviews: [
      { id: 'r1', author: 'Sarah M.', rating: 5, text: 'Saved me $120 on my internet bill in the first month!', date: '2026-02-15' },
      { id: 'r2', author: 'James K.', rating: 5, text: 'Works like magic. Set it and forget it.', date: '2026-01-28' },
      { id: 'r3', author: 'Lisa P.', rating: 4, text: 'Great for cable bills. Insurance negotiation could be better.', date: '2026-01-10' },
    ],
    featured: true,
    version: '3.2.1',
  },
  {
    id: 'agent-002',
    name: 'Health Guardian',
    publisher: 'VitalAI',
    domain: 'Health',
    description: 'Monitors your health trajectory and provides personalized insights.',
    longDescription:
      'Health Guardian connects to your wearables and health apps to build a comprehensive picture of your wellbeing. It tracks sleep patterns, activity levels, heart rate variability, and nutrition to detect trends before they become problems.\n\nThe agent provides weekly health reports with actionable recommendations tailored to your goals, whether you want to improve sleep, increase fitness, or manage stress.',
    rating: 4.7,
    ratingCount: 18200,
    installs: 890_000,
    price: 'Free',
    icon: 'monitor-heart',
    iconColor: '#ec4899',
    permissions: ['Access health data', 'Read wearable data', 'Send notifications'],
    reviews: [
      { id: 'r4', author: 'Mike R.', rating: 5, text: 'Caught an irregular heart pattern early. Literally a lifesaver.', date: '2026-03-01' },
      { id: 'r5', author: 'Anna S.', rating: 4, text: 'Love the weekly reports. Very insightful.', date: '2026-02-20' },
    ],
    featured: true,
    version: '2.8.0',
  },
  {
    id: 'agent-003',
    name: 'Legal Shield',
    publisher: 'LexAI Corp',
    domain: 'Legal',
    description: 'Reviews terms of service, contracts, and legal documents for hidden risks.',
    longDescription:
      'Legal Shield automatically scans terms of service, rental agreements, employment contracts, and other legal documents. It highlights concerning clauses, identifies unusual terms, and explains legal jargon in plain language.\n\nThe agent maintains a database of thousands of standard contract templates to compare your documents against industry norms.',
    rating: 4.6,
    ratingCount: 9800,
    installs: 450_000,
    price: 'Plus',
    icon: 'shield',
    iconColor: '#f59e0b',
    permissions: ['Read documents', 'Access camera for scanning', 'Store document history'],
    reviews: [
      { id: 'r6', author: 'David L.', rating: 5, text: 'Found a hidden auto-renewal clause in my gym contract.', date: '2026-02-10' },
    ],
    featured: false,
    version: '1.9.4',
  },
  {
    id: 'agent-004',
    name: 'Tax Advisor',
    publisher: 'RuVix Labs',
    domain: 'Finance',
    description: 'Smart tax optimization with year-round planning and deduction tracking.',
    longDescription:
      'Tax Advisor works year-round to optimize your tax situation. It tracks deductible expenses, suggests retirement account contributions, and models different filing strategies to minimize your tax burden.\n\nThe agent integrates with major financial institutions and provides quarterly estimated tax calculations for freelancers and business owners.',
    rating: 4.9,
    ratingCount: 42000,
    installs: 2_100_000,
    price: 'Pro',
    icon: 'calculate',
    iconColor: '#22c55e',
    permissions: ['Read financial accounts', 'Access tax documents', 'Read employment data'],
    reviews: [
      { id: 'r7', author: 'Chris W.', rating: 5, text: 'Found $3,200 in deductions I would have missed.', date: '2026-03-10' },
      { id: 'r8', author: 'Maria G.', rating: 5, text: 'Best tax tool I have ever used. Worth every penny.', date: '2026-02-28' },
    ],
    featured: true,
    version: '4.1.0',
  },
  {
    id: 'agent-005',
    name: 'Meal Planner',
    publisher: 'NutriBot',
    domain: 'Health',
    description: 'Personalized nutrition planning based on your health goals and preferences.',
    longDescription:
      'Meal Planner creates customized weekly meal plans based on your dietary needs, allergies, taste preferences, and health goals. It generates shopping lists, suggests recipes, and tracks macro and micronutrient intake.\n\nThe agent learns your preferences over time and adapts plans based on seasonal ingredient availability and local grocery prices.',
    rating: 4.5,
    ratingCount: 14300,
    installs: 670_000,
    price: 'Free',
    icon: 'restaurant',
    iconColor: '#ec4899',
    permissions: ['Access health data', 'Read location for grocery stores', 'Send notifications'],
    reviews: [
      { id: 'r9', author: 'Tom H.', rating: 4, text: 'Great recipes. Wish it had more vegan options.', date: '2026-01-15' },
    ],
    featured: false,
    version: '2.3.2',
  },
  {
    id: 'agent-006',
    name: 'Shopping Sentinel',
    publisher: 'DealHawk',
    domain: 'Shopping',
    description: 'Price tracking, deal alerts, and automatic coupon finding across stores.',
    longDescription:
      'Shopping Sentinel monitors prices across hundreds of retailers and alerts you when items on your wishlist drop in price. It automatically finds and applies coupon codes at checkout and compares prices across stores.\n\nThe agent tracks price history so you know if a "sale" is really a good deal. It also predicts optimal purchase timing based on historical pricing patterns.',
    rating: 4.8,
    ratingCount: 31200,
    installs: 1_500_000,
    price: 'Free',
    icon: 'local-offer',
    iconColor: '#f97316',
    permissions: ['Read browsing activity', 'Access notifications', 'Read purchase history'],
    reviews: [
      { id: 'r10', author: 'Emily R.', rating: 5, text: 'Saved $400 on Black Friday without lifting a finger.', date: '2026-01-05' },
    ],
    featured: true,
    version: '3.0.1',
  },
  {
    id: 'agent-007',
    name: 'Trip Planner',
    publisher: 'WanderAI',
    domain: 'Travel',
    description: 'Smart travel planning with itineraries, bookings, and local tips.',
    longDescription:
      'Trip Planner creates comprehensive travel itineraries based on your interests, budget, and travel style. It finds the best flights, hotels, and activities while considering your preferences for pace and adventure level.\n\nThe agent provides real-time travel alerts, suggests hidden gems based on local knowledge, and handles booking confirmations and schedule changes.',
    rating: 4.4,
    ratingCount: 7600,
    installs: 320_000,
    price: 'Plus',
    icon: 'explore',
    iconColor: '#14b8a6',
    permissions: ['Access calendar', 'Read location', 'Access payment methods'],
    reviews: [
      { id: 'r11', author: 'Jake M.', rating: 5, text: 'Planned my entire Japan trip. Every suggestion was perfect.', date: '2026-02-05' },
    ],
    featured: false,
    version: '1.7.3',
  },
  {
    id: 'agent-008',
    name: 'Career Coach',
    publisher: 'PathFinder AI',
    domain: 'Career',
    description: 'Resume optimization, interview prep, and salary negotiation guidance.',
    longDescription:
      'Career Coach analyzes your resume against job descriptions, suggests improvements, and prepares you for interviews with AI-powered mock sessions. It researches salary ranges for your role and location to help you negotiate better compensation.\n\nThe agent tracks job market trends in your field and alerts you to opportunities that match your career goals.',
    rating: 4.7,
    ratingCount: 15600,
    installs: 780_000,
    price: 'Plus',
    icon: 'trending-up',
    iconColor: '#3b82f6',
    permissions: ['Read documents', 'Access calendar', 'Read professional profiles'],
    reviews: [
      { id: 'r12', author: 'Priya S.', rating: 5, text: 'Got a 25% raise after using the salary negotiation feature.', date: '2026-03-05' },
    ],
    featured: true,
    version: '2.5.0',
  },
  {
    id: 'agent-009',
    name: 'Study Buddy',
    publisher: 'EduTech AI',
    domain: 'Education',
    description: 'Adaptive learning assistant with spaced repetition and concept mapping.',
    longDescription:
      'Study Buddy uses spaced repetition algorithms and concept mapping to help you learn and retain information more effectively. It creates personalized study schedules, generates practice questions, and identifies knowledge gaps.\n\nThe agent supports multiple subjects from languages to sciences and adapts its teaching style to your learning preferences.',
    rating: 4.6,
    ratingCount: 21000,
    installs: 920_000,
    price: 'Free',
    icon: 'auto-stories',
    iconColor: '#8b5cf6',
    permissions: ['Access documents', 'Send notifications', 'Store study data'],
    reviews: [
      { id: 'r13', author: 'Alex T.', rating: 5, text: 'Aced my bar exam prep. The spaced repetition is incredible.', date: '2026-01-20' },
    ],
    featured: false,
    version: '3.1.0',
  },
  {
    id: 'agent-010',
    name: 'Home Manager',
    publisher: 'NestAI',
    domain: 'Home',
    description: 'Smart home coordination, maintenance scheduling, and energy optimization.',
    longDescription:
      'Home Manager coordinates your smart home devices, schedules preventive maintenance, and optimizes energy usage to reduce utility bills. It tracks warranty expirations, suggests seasonal maintenance tasks, and manages contractor contacts.\n\nThe agent integrates with major smart home platforms and provides energy usage analytics with personalized savings recommendations.',
    rating: 4.3,
    ratingCount: 8900,
    installs: 340_000,
    price: 'Free',
    icon: 'smart-toy',
    iconColor: '#06b6d4',
    permissions: ['Access smart home devices', 'Read energy data', 'Send notifications', 'Access calendar'],
    reviews: [
      { id: 'r14', author: 'Robert K.', rating: 4, text: 'Reduced my energy bill by 18%. Maintenance reminders are handy.', date: '2026-02-12' },
    ],
    featured: false,
    version: '1.4.2',
  },
  {
    id: 'agent-011',
    name: 'Social Planner',
    publisher: 'ConnectAI',
    domain: 'Social',
    description: 'Event planning, group coordination, and relationship maintenance reminders.',
    longDescription:
      'Social Planner helps you maintain meaningful relationships by suggesting meetups, remembering important dates, and coordinating group events. It finds venues, manages RSVPs, and handles scheduling conflicts.\n\nThe agent gently reminds you to reach out to contacts you have not connected with recently and suggests conversation starters based on shared interests.',
    rating: 4.2,
    ratingCount: 5400,
    installs: 210_000,
    price: 'Free',
    icon: 'groups',
    iconColor: '#a855f7',
    permissions: ['Access contacts', 'Access calendar', 'Read location', 'Send messages'],
    reviews: [
      { id: 'r15', author: 'Nina P.', rating: 4, text: 'Love the birthday and anniversary reminders. Event planning is smooth.', date: '2026-01-30' },
    ],
    featured: false,
    version: '1.2.1',
  },
  {
    id: 'agent-012',
    name: 'Gov Navigator',
    publisher: 'CivicTech',
    domain: 'Government',
    description: 'Navigates government forms, deadlines, and benefit eligibility.',
    longDescription:
      'Gov Navigator simplifies interactions with government agencies. It tracks deadlines for license renewals, tax filings, and permit applications. It determines your eligibility for benefits and assistance programs and guides you through application processes.\n\nThe agent maintains a database of federal, state, and local programs and can pre-fill forms with your stored information.',
    rating: 4.5,
    ratingCount: 6700,
    installs: 280_000,
    price: 'Free',
    icon: 'assured-workload',
    iconColor: '#64748b',
    permissions: ['Read personal documents', 'Access calendar', 'Store form data'],
    reviews: [
      { id: 'r16', author: 'Carlos M.', rating: 5, text: 'Found a property tax exemption I did not know I qualified for.', date: '2026-02-18' },
    ],
    featured: false,
    version: '1.6.0',
  },
  {
    id: 'agent-013',
    name: 'Auto Mechanic',
    publisher: 'DriveAI',
    domain: 'Automotive',
    description: 'Vehicle maintenance tracking, repair cost estimates, and recall alerts.',
    longDescription:
      'Auto Mechanic tracks your vehicle maintenance schedule, provides repair cost estimates from local shops, and alerts you to safety recalls. It logs service history, monitors fluid levels through OBD-II connections, and predicts component failures.\n\nThe agent compares shop rates in your area and can schedule appointments at trusted mechanics.',
    rating: 4.4,
    ratingCount: 11200,
    installs: 520_000,
    price: 'Free',
    icon: 'build',
    iconColor: '#ef4444',
    permissions: ['Access vehicle data', 'Read location', 'Send notifications'],
    reviews: [
      { id: 'r17', author: 'Steve B.', rating: 5, text: 'Saved me from a bad transmission repair quote. Found a shop $800 cheaper.', date: '2026-03-08' },
    ],
    featured: false,
    version: '2.1.0',
  },
  {
    id: 'agent-014',
    name: 'Pet Companion',
    publisher: 'PawPal AI',
    domain: 'Pet',
    description: 'Pet health tracking, vet scheduling, and breed-specific care guidance.',
    longDescription:
      'Pet Companion monitors your pet health milestones, vaccination schedules, and dietary needs. It provides breed-specific care guidance, tracks medications, and helps you find emergency vet services nearby.\n\nThe agent can identify potential health issues from behavioral changes you log and connects you with veterinary telehealth services for quick consultations.',
    rating: 4.7,
    ratingCount: 13400,
    installs: 610_000,
    price: 'Free',
    icon: 'pets',
    iconColor: '#84cc16',
    permissions: ['Access calendar', 'Read location', 'Send notifications', 'Access camera'],
    reviews: [
      { id: 'r18', author: 'Laura W.', rating: 5, text: 'Noticed my dog was drinking more water than usual. Vet confirmed early kidney issue.', date: '2026-02-22' },
    ],
    featured: false,
    version: '1.8.1',
  },
  {
    id: 'agent-015',
    name: 'Investment Analyst',
    publisher: 'WealthEngine',
    domain: 'Finance',
    description: 'Portfolio analysis, market insights, and rebalancing recommendations.',
    longDescription:
      'Investment Analyst provides professional-grade portfolio analysis for individual investors. It monitors your holdings, suggests rebalancing opportunities, and provides market insights tailored to your investment strategy.\n\nThe agent tracks dividend income, capital gains, and tax-loss harvesting opportunities. It backtests strategies and provides risk-adjusted return comparisons.',
    rating: 4.8,
    ratingCount: 19800,
    installs: 1_340_000,
    price: 'Pro',
    icon: 'candlestick-chart',
    iconColor: '#22c55e',
    permissions: ['Read financial accounts', 'Access market data', 'Send notifications'],
    reviews: [
      { id: 'r19', author: 'Daniel F.', rating: 5, text: 'Tax-loss harvesting alone paid for the Pro subscription ten times over.', date: '2026-03-12' },
    ],
    featured: true,
    version: '3.5.0',
  },
  {
    id: 'agent-016',
    name: 'Sleep Optimizer',
    publisher: 'DreamTech',
    domain: 'Health',
    description: 'Sleep analysis, circadian rhythm optimization, and environment tuning.',
    longDescription:
      'Sleep Optimizer analyzes your sleep patterns using wearable data and provides actionable recommendations to improve sleep quality. It adjusts smart home lighting and temperature based on your circadian rhythm.\n\nThe agent tracks sleep debt, suggests optimal bedtimes, and can generate personalized wind-down routines with guided relaxation exercises.',
    rating: 4.6,
    ratingCount: 16100,
    installs: 740_000,
    price: 'Plus',
    icon: 'bedtime',
    iconColor: '#ec4899',
    permissions: ['Access health data', 'Access smart home devices', 'Read wearable data'],
    reviews: [
      { id: 'r20', author: 'Sophie L.', rating: 5, text: 'My sleep score improved from 62 to 89 in three weeks.', date: '2026-02-25' },
    ],
    featured: false,
    version: '2.0.3',
  },
  {
    id: 'agent-017',
    name: 'Contract Drafter',
    publisher: 'LexAI Corp',
    domain: 'Legal',
    description: 'Generates and reviews freelance contracts, NDAs, and lease agreements.',
    longDescription:
      'Contract Drafter creates legally sound documents from templates customized to your jurisdiction. It supports freelance agreements, NDAs, lease agreements, and service contracts with clause-by-clause explanations.\n\nThe agent can compare your contracts against legal databases to ensure market-standard terms and flag unusual provisions.',
    rating: 4.5,
    ratingCount: 7200,
    installs: 310_000,
    price: 'Pro',
    icon: 'description',
    iconColor: '#f59e0b',
    permissions: ['Read documents', 'Store document history', 'Access templates'],
    reviews: [
      { id: 'r21', author: 'Freelancer Joe', rating: 4, text: 'Good templates. The NDA generator saved me hours.', date: '2026-01-25' },
    ],
    featured: false,
    version: '1.5.2',
  },
  {
    id: 'agent-018',
    name: 'Language Tutor',
    publisher: 'PolyglotAI',
    domain: 'Education',
    description: 'Conversational language learning with pronunciation feedback.',
    longDescription:
      'Language Tutor provides immersive language learning through AI-powered conversations. It offers pronunciation feedback using speech recognition, grammar correction in context, and vocabulary building through spaced repetition.\n\nThe agent adapts to your proficiency level and learning pace, covering 40+ languages with culturally relevant content and real-world scenarios.',
    rating: 4.7,
    ratingCount: 28500,
    installs: 1_100_000,
    price: 'Plus',
    icon: 'translate',
    iconColor: '#8b5cf6',
    permissions: ['Access microphone', 'Send notifications', 'Store learning data'],
    reviews: [
      { id: 'r22', author: 'Yuki T.', rating: 5, text: 'Better than any language app I have tried. Conversations feel natural.', date: '2026-03-02' },
    ],
    featured: true,
    version: '4.0.1',
  },
  {
    id: 'agent-019',
    name: 'Budget Tracker',
    publisher: 'PennyWise',
    domain: 'Finance',
    description: 'Automatic expense categorization and spending insights.',
    longDescription:
      'Budget Tracker automatically categorizes your transactions, identifies spending patterns, and provides insights to help you stick to your budget. It supports multiple accounts, shared budgets for families, and custom category rules.\n\nThe agent provides weekly spending summaries, alerts when you approach budget limits, and suggests areas where you could save based on your transaction history.',
    rating: 4.6,
    ratingCount: 22100,
    installs: 980_000,
    price: 'Free',
    icon: 'account-balance-wallet',
    iconColor: '#22c55e',
    permissions: ['Read financial accounts', 'Send notifications'],
    reviews: [
      { id: 'r23', author: 'Rachel D.', rating: 4, text: 'Finally a budget app that actually categorizes things correctly.', date: '2026-02-08' },
    ],
    featured: false,
    version: '2.9.0',
  },
  {
    id: 'agent-020',
    name: 'Home Security',
    publisher: 'GuardianAI',
    domain: 'Home',
    description: 'Smart security monitoring with anomaly detection and emergency response.',
    longDescription:
      'Home Security integrates with your cameras, door locks, and sensors to provide intelligent security monitoring. It uses computer vision to distinguish between normal activity, deliveries, and genuine threats.\n\nThe agent can automatically lock doors, trigger alarms, contact emergency services, and notify trusted contacts when it detects suspicious activity.',
    rating: 4.8,
    ratingCount: 17500,
    installs: 860_000,
    price: 'Pro',
    icon: 'security',
    iconColor: '#06b6d4',
    permissions: ['Access cameras', 'Access smart home devices', 'Send notifications', 'Access contacts'],
    reviews: [
      { id: 'r24', author: 'Mark J.', rating: 5, text: 'Detected a package thief and captured clear footage. Police caught them.', date: '2026-03-15' },
    ],
    featured: false,
    version: '2.2.1',
  },
  {
    id: 'agent-021',
    name: 'Fitness Coach',
    publisher: 'IronAI',
    domain: 'Health',
    description: 'Personalized workout plans with form tracking and progressive overload.',
    longDescription:
      'Fitness Coach creates customized workout programs based on your goals, equipment access, and fitness level. It tracks progressive overload, suggests deload weeks, and provides exercise form guidance through video analysis.\n\nThe agent adapts plans based on recovery data from your wearables and can modify workouts on the fly if you are short on time or energy.',
    rating: 4.5,
    ratingCount: 19200,
    installs: 830_000,
    price: 'Plus',
    icon: 'fitness-center',
    iconColor: '#ec4899',
    permissions: ['Access health data', 'Access camera', 'Read wearable data', 'Send notifications'],
    reviews: [
      { id: 'r25', author: 'Tyler B.', rating: 5, text: 'Gained 15 lbs of muscle in 6 months. Best trainer I have had.', date: '2026-02-14' },
    ],
    featured: false,
    version: '3.3.0',
  },
  {
    id: 'agent-022',
    name: 'Parking Finder',
    publisher: 'DriveAI',
    domain: 'Automotive',
    description: 'Real-time parking availability and street cleaning alerts.',
    longDescription:
      'Parking Finder uses crowd-sourced data and city APIs to show real-time parking availability near your destination. It tracks street cleaning schedules, meter expirations, and parking garage rates.\n\nThe agent remembers where you parked, sets meter reminders, and can even find the cheapest long-term parking options for airports and events.',
    rating: 4.3,
    ratingCount: 8400,
    installs: 390_000,
    price: 'Free',
    icon: 'local-parking',
    iconColor: '#ef4444',
    permissions: ['Read location', 'Send notifications', 'Access calendar'],
    reviews: [
      { id: 'r26', author: 'City Driver', rating: 4, text: 'The street cleaning alerts alone saved me hundreds in tickets.', date: '2026-01-18' },
    ],
    featured: false,
    version: '1.3.0',
  },
  {
    id: 'agent-023',
    name: 'Pet Sitter Match',
    publisher: 'PawPal AI',
    domain: 'Pet',
    description: 'Connects you with vetted pet sitters and dog walkers in your area.',
    longDescription:
      'Pet Sitter Match helps you find trusted, vetted pet sitters and dog walkers nearby. It manages booking, payment, and provides GPS tracking during walks. Sitters send photo updates so you always know your pet is happy.\n\nThe agent matches you with sitters based on your pet breed, temperament, and specific care needs.',
    rating: 4.4,
    ratingCount: 6100,
    installs: 250_000,
    price: 'Free',
    icon: 'volunteer-activism',
    iconColor: '#84cc16',
    permissions: ['Read location', 'Access contacts', 'Access payment methods'],
    reviews: [
      { id: 'r27', author: 'Dog Mom', rating: 5, text: 'Found an amazing walker for my anxious rescue. He loves her!', date: '2026-03-01' },
    ],
    featured: false,
    version: '1.1.0',
  },
  {
    id: 'agent-024',
    name: 'Scholarship Scout',
    publisher: 'EduTech AI',
    domain: 'Education',
    description: 'Finds scholarships and grants matching your profile and goals.',
    longDescription:
      'Scholarship Scout scans thousands of scholarship databases to find opportunities matching your academic profile, interests, and demographics. It tracks deadlines, helps with application essays, and manages submission workflows.\n\nThe agent continuously monitors for new scholarships and updates your matches as your profile evolves.',
    rating: 4.8,
    ratingCount: 12300,
    installs: 560_000,
    price: 'Free',
    icon: 'emoji-events',
    iconColor: '#8b5cf6',
    permissions: ['Read academic records', 'Access documents', 'Send notifications'],
    reviews: [
      { id: 'r28', author: 'Student A.', rating: 5, text: 'Won a $5,000 scholarship I never would have found on my own.', date: '2026-02-28' },
    ],
    featured: false,
    version: '2.0.0',
  },
  {
    id: 'agent-025',
    name: 'Visa Advisor',
    publisher: 'WanderAI',
    domain: 'Travel',
    description: 'Visa requirements, application tracking, and travel document management.',
    longDescription:
      'Visa Advisor determines visa requirements for your nationality and destination, guides you through application processes, and tracks approval status. It manages passport expiration dates and entry/exit requirements.\n\nThe agent stores digital copies of travel documents and provides offline access to visa information and emergency contacts.',
    rating: 4.5,
    ratingCount: 5800,
    installs: 230_000,
    price: 'Plus',
    icon: 'badge',
    iconColor: '#14b8a6',
    permissions: ['Read personal documents', 'Access camera', 'Store document copies'],
    reviews: [
      { id: 'r29', author: 'Global Nomad', rating: 5, text: 'Made my multi-country visa applications painless.', date: '2026-01-12' },
    ],
    featured: false,
    version: '1.4.1',
  },
];

// Generate the full collection of 52 agents (7 unlocked, 45 locked)
function generateCollectedAgents(): CollectedAgent[] {
  const unlocked: CollectedAgent[] = [
    {
      id: 'col-001',
      marketplaceId: 'agent-001',
      name: 'Bill Negotiator',
      domain: 'Finance',
      icon: 'receipt-long',
      iconColor: '#22c55e',
      level: 6,
      xp: 1720,
      xpToNext: 2200,
      unlocked: true,
      lastUsed: '2026-03-19',
    },
    {
      id: 'col-002',
      marketplaceId: 'agent-002',
      name: 'Health Guardian',
      domain: 'Health',
      icon: 'monitor-heart',
      iconColor: '#ec4899',
      level: 4,
      xp: 840,
      xpToNext: 1000,
      unlocked: true,
      lastUsed: '2026-03-18',
    },
    {
      id: 'col-004',
      marketplaceId: 'agent-004',
      name: 'Tax Advisor',
      domain: 'Finance',
      icon: 'calculate',
      iconColor: '#22c55e',
      level: 8,
      xp: 3600,
      xpToNext: 4000,
      unlocked: true,
      lastUsed: '2026-03-17',
    },
    {
      id: 'col-006',
      marketplaceId: 'agent-006',
      name: 'Shopping Sentinel',
      domain: 'Shopping',
      icon: 'local-offer',
      iconColor: '#f97316',
      level: 3,
      xp: 450,
      xpToNext: 600,
      unlocked: true,
      lastUsed: '2026-03-16',
    },
    {
      id: 'col-009',
      marketplaceId: 'agent-009',
      name: 'Study Buddy',
      domain: 'Education',
      icon: 'auto-stories',
      iconColor: '#8b5cf6',
      level: 5,
      xp: 1200,
      xpToNext: 1500,
      unlocked: true,
      lastUsed: '2026-03-15',
    },
    {
      id: 'col-014',
      marketplaceId: 'agent-014',
      name: 'Pet Companion',
      domain: 'Pet',
      icon: 'pets',
      iconColor: '#84cc16',
      level: 2,
      xp: 180,
      xpToNext: 300,
      unlocked: true,
      lastUsed: '2026-03-10',
    },
    {
      id: 'col-019',
      marketplaceId: 'agent-019',
      name: 'Budget Tracker',
      domain: 'Finance',
      icon: 'account-balance-wallet',
      iconColor: '#22c55e',
      level: 7,
      xp: 2800,
      xpToNext: 3000,
      unlocked: true,
      lastUsed: '2026-03-19',
    },
  ];

  const lockedNames = [
    { name: 'Legal Shield', domain: 'Legal' as Domain, icon: 'shield', iconColor: '#f59e0b' },
    { name: 'Meal Planner', domain: 'Health' as Domain, icon: 'restaurant', iconColor: '#ec4899' },
    { name: 'Trip Planner', domain: 'Travel' as Domain, icon: 'explore', iconColor: '#14b8a6' },
    { name: 'Career Coach', domain: 'Career' as Domain, icon: 'trending-up', iconColor: '#3b82f6' },
    { name: 'Home Manager', domain: 'Home' as Domain, icon: 'smart-toy', iconColor: '#06b6d4' },
    { name: 'Social Planner', domain: 'Social' as Domain, icon: 'groups', iconColor: '#a855f7' },
    { name: 'Gov Navigator', domain: 'Government' as Domain, icon: 'assured-workload', iconColor: '#64748b' },
    { name: 'Auto Mechanic', domain: 'Automotive' as Domain, icon: 'build', iconColor: '#ef4444' },
    { name: 'Investment Analyst', domain: 'Finance' as Domain, icon: 'candlestick-chart', iconColor: '#22c55e' },
    { name: 'Sleep Optimizer', domain: 'Health' as Domain, icon: 'bedtime', iconColor: '#ec4899' },
    { name: 'Contract Drafter', domain: 'Legal' as Domain, icon: 'description', iconColor: '#f59e0b' },
    { name: 'Language Tutor', domain: 'Education' as Domain, icon: 'translate', iconColor: '#8b5cf6' },
    { name: 'Home Security', domain: 'Home' as Domain, icon: 'security', iconColor: '#06b6d4' },
    { name: 'Fitness Coach', domain: 'Health' as Domain, icon: 'fitness-center', iconColor: '#ec4899' },
    { name: 'Parking Finder', domain: 'Automotive' as Domain, icon: 'local-parking', iconColor: '#ef4444' },
    { name: 'Pet Sitter Match', domain: 'Pet' as Domain, icon: 'volunteer-activism', iconColor: '#84cc16' },
    { name: 'Scholarship Scout', domain: 'Education' as Domain, icon: 'emoji-events', iconColor: '#8b5cf6' },
    { name: 'Visa Advisor', domain: 'Travel' as Domain, icon: 'badge', iconColor: '#14b8a6' },
    { name: 'Debt Destroyer', domain: 'Finance' as Domain, icon: 'money-off', iconColor: '#22c55e' },
    { name: 'Mindfulness Guide', domain: 'Health' as Domain, icon: 'self-improvement', iconColor: '#ec4899' },
    { name: 'Estate Planner', domain: 'Legal' as Domain, icon: 'villa', iconColor: '#f59e0b' },
    { name: 'Resume Builder', domain: 'Career' as Domain, icon: 'article', iconColor: '#3b82f6' },
    { name: 'Math Solver', domain: 'Education' as Domain, icon: 'functions', iconColor: '#8b5cf6' },
    { name: 'Lawn Care', domain: 'Home' as Domain, icon: 'grass', iconColor: '#06b6d4' },
    { name: 'Wardrobe Stylist', domain: 'Shopping' as Domain, icon: 'checkroom', iconColor: '#f97316' },
    { name: 'Road Trip DJ', domain: 'Travel' as Domain, icon: 'music-note', iconColor: '#14b8a6' },
    { name: 'Event Coordinator', domain: 'Social' as Domain, icon: 'celebration', iconColor: '#a855f7' },
    { name: 'Tax Filing', domain: 'Government' as Domain, icon: 'receipt', iconColor: '#64748b' },
    { name: 'EV Charger Finder', domain: 'Automotive' as Domain, icon: 'ev-station', iconColor: '#ef4444' },
    { name: 'Pet Trainer', domain: 'Pet' as Domain, icon: 'school', iconColor: '#84cc16' },
    { name: 'Crypto Watch', domain: 'Finance' as Domain, icon: 'currency-bitcoin', iconColor: '#22c55e' },
    { name: 'Allergy Tracker', domain: 'Health' as Domain, icon: 'healing', iconColor: '#ec4899' },
    { name: 'Dispute Resolver', domain: 'Legal' as Domain, icon: 'balance', iconColor: '#f59e0b' },
    { name: 'Freelance Manager', domain: 'Career' as Domain, icon: 'laptop', iconColor: '#3b82f6' },
    { name: 'Thesis Helper', domain: 'Education' as Domain, icon: 'history-edu', iconColor: '#8b5cf6' },
    { name: 'Appliance Doctor', domain: 'Home' as Domain, icon: 'kitchen', iconColor: '#06b6d4' },
    { name: 'Grocery Optimizer', domain: 'Shopping' as Domain, icon: 'shopping-basket', iconColor: '#f97316' },
    { name: 'Flight Tracker', domain: 'Travel' as Domain, icon: 'flight-takeoff', iconColor: '#14b8a6' },
    { name: 'Dating Coach', domain: 'Social' as Domain, icon: 'favorite-border', iconColor: '#a855f7' },
    { name: 'Benefits Finder', domain: 'Government' as Domain, icon: 'policy', iconColor: '#64748b' },
    { name: 'Insurance Advisor', domain: 'Automotive' as Domain, icon: 'verified-user', iconColor: '#ef4444' },
    { name: 'Vet Telehealth', domain: 'Pet' as Domain, icon: 'video-call', iconColor: '#84cc16' },
    { name: 'Rent Negotiator', domain: 'Finance' as Domain, icon: 'apartment', iconColor: '#22c55e' },
    { name: 'Posture Coach', domain: 'Health' as Domain, icon: 'accessibility-new', iconColor: '#ec4899' },
    { name: 'Parking Ticket Fighter', domain: 'Legal' as Domain, icon: 'gavel', iconColor: '#f59e0b' },
  ];

  const locked: CollectedAgent[] = lockedNames.map((agent, i) => ({
    id: `col-locked-${String(i + 1).padStart(3, '0')}`,
    marketplaceId: `agent-locked-${i + 1}`,
    name: agent.name,
    domain: agent.domain,
    icon: agent.icon,
    iconColor: agent.iconColor,
    level: 0,
    xp: 0,
    xpToNext: 100,
    unlocked: false,
  }));

  return [...unlocked, ...locked];
}

export const COLLECTED_AGENTS = generateCollectedAgents();

export const COLLECTION_STATS = {
  collected: 7,
  total: 52,
  totalLevel: 14,
  totalXp: 2847,
};
