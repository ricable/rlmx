/**
 * useVoice — React hook for voice interaction in the RuVix mobile app.
 *
 * Manages listening state, transcripts, agent progress, and response cards.
 * Defaults to demo mode simulation; switches to live mode automatically
 * when the RLMX server is detected.
 */

import { useCallback, useRef, useState } from 'react';
import {
  AgentProgressUpdate,
  getVoiceService,
  VoiceResponse,
  VoiceService,
} from '../services/voice';

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export interface UseVoiceResult {
  isListening: boolean;
  transcript: string;
  agentProgress: AgentProgressUpdate[];
  responses: VoiceResponse[];
  isDemoMode: boolean;
  startListening: () => void;
  stopListening: () => void;
  simulateQuery: (text: string) => Promise<void>;
  clearResponses: () => void;
}

// ---------------------------------------------------------------------------
// Hook
// ---------------------------------------------------------------------------

export function useVoice(serverUrl?: string): UseVoiceResult {
  const serviceRef = useRef<VoiceService>(getVoiceService(serverUrl));
  const [isListening, setIsListening] = useState(false);
  const [transcript, setTranscript] = useState('');
  const [agentProgress, setAgentProgress] = useState<AgentProgressUpdate[]>([]);
  const [responses, setResponses] = useState<VoiceResponse[]>([]);

  const startListening = useCallback(async () => {
    setIsListening(true);
    setAgentProgress([]);
    setResponses([]);
    await serviceRef.current.startListening();
  }, []);

  const stopListening = useCallback(async () => {
    const text = await serviceRef.current.stopListening();
    setIsListening(false);

    if (text) {
      setTranscript(text);
      const result = await serviceRef.current.simulateVoiceInteraction(
        text,
        (update) => {
          setAgentProgress((prev) => {
            // Replace previous entry for the same domain + status level
            const existing = prev.findIndex(
              (p) => p.domain === update.domain && p.status === update.status,
            );
            if (existing >= 0) {
              const next = [...prev];
              next[existing] = update;
              return next;
            }
            return [...prev, update];
          });
        },
      );
      setResponses(result.responses);
    }
  }, []);

  const simulateQuery = useCallback(async (text: string) => {
    setTranscript(text);
    setIsListening(true);
    setAgentProgress([]);
    setResponses([]);

    // Brief delay to show "listening" state
    await new Promise<void>((resolve) => setTimeout(resolve, 500));
    setIsListening(false);

    const result = await serviceRef.current.simulateVoiceInteraction(
      text,
      (update) => {
        setAgentProgress((prev) => [...prev, update]);
      },
    );
    setResponses(result.responses);
  }, []);

  const clearResponses = useCallback(() => {
    setTranscript('');
    setAgentProgress([]);
    setResponses([]);
  }, []);

  return {
    isListening,
    transcript,
    agentProgress,
    responses,
    isDemoMode: serviceRef.current.isDemoMode(),
    startListening,
    stopListening,
    simulateQuery,
    clearResponses,
  };
}
