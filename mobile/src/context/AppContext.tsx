import React, {createContext, useContext, useReducer, useEffect} from 'react';
import type {AppState, VoiceMessage, AgentProgress} from '../types';
import {
  demoUser,
  demoAgents,
  demoDomainScores,
  demoBriefing,
  demoAchievements,
  demoVoiceProgress,
  computeLifeScore,
} from '../services/demo';
import {checkConnection} from '../services/api';

type Action =
  | {type: 'SET_LISTENING'; payload: boolean}
  | {type: 'ADD_VOICE_MESSAGE'; payload: VoiceMessage}
  | {type: 'SET_AGENT_PROGRESS'; payload: AgentProgress[]}
  | {type: 'SET_CONNECTED'; payload: boolean}
  | {type: 'CLEAR_VOICE'};

const initialState: AppState = {
  user: demoUser,
  agents: demoAgents,
  lifeScore: computeLifeScore(demoDomainScores),
  domainScores: demoDomainScores,
  moneySaved: 847,
  briefing: demoBriefing,
  achievements: demoAchievements,
  voiceMessages: [],
  agentProgress: [],
  isListening: false,
  isConnected: false,
};

function reducer(state: AppState, action: Action): AppState {
  switch (action.type) {
    case 'SET_LISTENING':
      return {...state, isListening: action.payload};
    case 'ADD_VOICE_MESSAGE':
      return {...state, voiceMessages: [...state.voiceMessages, action.payload]};
    case 'SET_AGENT_PROGRESS':
      return {...state, agentProgress: action.payload};
    case 'SET_CONNECTED':
      return {...state, isConnected: action.payload};
    case 'CLEAR_VOICE':
      return {...state, voiceMessages: [], agentProgress: []};
    default:
      return state;
  }
}

interface AppContextValue {
  state: AppState;
  dispatch: React.Dispatch<Action>;
  simulateVoiceQuery: (transcript: string) => void;
}

const AppContext = createContext<AppContextValue | null>(null);

export function AppProvider({children}: {children: React.ReactNode}) {
  const [state, dispatch] = useReducer(reducer, initialState);

  useEffect(() => {
    checkConnection().then(connected => {
      dispatch({type: 'SET_CONNECTED', payload: connected});
    });
  }, []);

  const simulateVoiceQuery = (transcript: string) => {
    dispatch({
      type: 'ADD_VOICE_MESSAGE',
      payload: {id: Date.now().toString(), role: 'user', text: transcript, timestamp: Date.now()},
    });

    dispatch({type: 'SET_AGENT_PROGRESS', payload: demoVoiceProgress.map(p => ({...p, status: 'working' as const, progress: 0}))});

    // Simulate agents working
    let tick = 0;
    const interval = setInterval(() => {
      tick++;
      dispatch({
        type: 'SET_AGENT_PROGRESS',
        payload: demoVoiceProgress.map((p, i) => ({
          ...p,
          status: tick > (i + 1) * 2 ? 'complete' as const : 'working' as const,
          progress: Math.min(100, tick * (15 - i * 2)),
        })),
      });
      if (tick >= 10) {
        clearInterval(interval);
        dispatch({
          type: 'ADD_VOICE_MESSAGE',
          payload: {
            id: (Date.now() + 1).toString(),
            role: 'assistant',
            text: 'I found 2 subscriptions you can cancel to save $47/month. I also optimized your Thursday schedule for 2 extra hours of focus time. Sentinel is still scanning breach databases.',
            timestamp: Date.now(),
          },
        });
      }
    }, 400);
  };

  return (
    <AppContext.Provider value={{state, dispatch, simulateVoiceQuery}}>
      {children}
    </AppContext.Provider>
  );
}

export function useApp(): AppContextValue {
  const ctx = useContext(AppContext);
  if (!ctx) {throw new Error('useApp must be used within AppProvider');}
  return ctx;
}
