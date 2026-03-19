import React, {useState, useEffect, useRef} from 'react';
import {
  View,
  Text,
  ScrollView,
  StyleSheet,
  TextInput,
  TouchableOpacity,
  KeyboardAvoidingView,
  Platform,
} from 'react-native';
import Icon from 'react-native-vector-icons/MaterialIcons';
import {colors, spacing, radius, fontSize} from '../theme';
import {useApp} from '../context/AppContext';
import VoiceMicButton from '../components/VoiceMicButton';
import ProgressCard from '../components/ProgressCard';
import {initVoice, startListening, stopListening, cleanup} from '../services/voice';

export default function VoiceScreen() {
  const {state, dispatch, simulateVoiceQuery} = useApp();
  const [transcript, setTranscript] = useState('');
  const [mode, setMode] = useState<'voice' | 'text'>('voice');
  const [textInput, setTextInput] = useState('');
  const scrollRef = useRef<ScrollView>(null);

  useEffect(() => {
    initVoice({
      onTranscript: (text) => setTranscript(text),
      onListeningChange: (listening) => {
        dispatch({type: 'SET_LISTENING', payload: listening});
        if (!listening && transcript.length > 5) {
          simulateVoiceQuery(transcript);
          setTranscript('');
        }
      },
      onError: () => {},
    });
    return cleanup;
  }, [transcript, dispatch, simulateVoiceQuery]);

  const handleMicPress = () => {
    if (state.isListening) {
      stopListening();
    } else {
      dispatch({type: 'CLEAR_VOICE'});
      startListening();
    }
  };

  const handleTextSubmit = () => {
    if (textInput.trim()) {
      dispatch({type: 'CLEAR_VOICE'});
      simulateVoiceQuery(textInput.trim());
      setTextInput('');
    }
  };

  return (
    <KeyboardAvoidingView
      style={styles.container}
      behavior={Platform.OS === 'ios' ? 'padding' : undefined}>
      <View style={styles.header}>
        <Text style={styles.title}>RuVix</Text>
        <View style={styles.modeToggle}>
          <TouchableOpacity
            style={[styles.modeBtn, mode === 'voice' && styles.modeBtnActive]}
            onPress={() => setMode('voice')}>
            <Icon name="mic" size={16} color={mode === 'voice' ? colors.white : colors.textMuted} />
            <Text style={[styles.modeBtnText, mode === 'voice' && styles.modeBtnTextActive]}>Voice</Text>
          </TouchableOpacity>
          <TouchableOpacity
            style={[styles.modeBtn, mode === 'text' && styles.modeBtnActive]}
            onPress={() => setMode('text')}>
            <Icon name="keyboard" size={16} color={mode === 'text' ? colors.white : colors.textMuted} />
            <Text style={[styles.modeBtnText, mode === 'text' && styles.modeBtnTextActive]}>Text</Text>
          </TouchableOpacity>
        </View>
      </View>

      <ScrollView
        ref={scrollRef}
        style={styles.messageArea}
        contentContainerStyle={styles.messageContent}
        onContentSizeChange={() => scrollRef.current?.scrollToEnd({animated: true})}>

        {state.voiceMessages.length === 0 && !state.isListening && (
          <View style={styles.emptyState}>
            <View style={styles.emptyIcon}>
              <Icon name="auto-awesome" size={48} color={colors.cyan} />
            </View>
            <Text style={styles.emptyTitle}>Talk to your agents</Text>
            <Text style={styles.emptySubtitle}>
              Tap the mic and ask anything. Your AI agents will work together to help you.
            </Text>
            <View style={styles.suggestions}>
              {['Check my spending', 'Optimize my schedule', 'Security checkup'].map(s => (
                <TouchableOpacity
                  key={s}
                  style={styles.suggestion}
                  onPress={() => {
                    dispatch({type: 'CLEAR_VOICE'});
                    simulateVoiceQuery(s);
                  }}>
                  <Text style={styles.suggestionText}>{s}</Text>
                  <Icon name="arrow-forward" size={14} color={colors.cyan} />
                </TouchableOpacity>
              ))}
            </View>
          </View>
        )}

        {/* Transcript */}
        {state.isListening && transcript.length > 0 && (
          <View style={styles.transcriptContainer}>
            <View style={styles.transcriptDot} />
            <Text style={styles.transcript}>{transcript}</Text>
          </View>
        )}

        {/* Messages */}
        {state.voiceMessages.map(msg => (
          <View
            key={msg.id}
            style={[
              styles.messageBubble,
              msg.role === 'user' ? styles.userBubble : styles.assistantBubble,
            ]}>
            {msg.role === 'assistant' && (
              <Icon name="auto-awesome" size={16} color={colors.cyan} style={styles.assistantIcon} />
            )}
            <Text style={[styles.messageText, msg.role === 'user' && styles.userMessageText]}>
              {msg.text}
            </Text>
          </View>
        ))}

        {/* Agent Progress */}
        {state.agentProgress.length > 0 && (
          <View style={styles.progressSection}>
            <Text style={styles.progressTitle}>Agents working</Text>
            <View style={styles.progressGrid}>
              {state.agentProgress.map(p => (
                <ProgressCard key={p.agentId} progress={p} />
              ))}
            </View>
          </View>
        )}
      </ScrollView>

      {/* Bottom input area */}
      <View style={styles.inputArea}>
        {mode === 'voice' ? (
          <View style={styles.voiceArea}>
            {state.isListening && (
              <Text style={styles.listeningText}>Listening...</Text>
            )}
            <VoiceMicButton isListening={state.isListening} onPress={handleMicPress} />
          </View>
        ) : (
          <View style={styles.textArea}>
            <TextInput
              style={styles.textInput}
              value={textInput}
              onChangeText={setTextInput}
              placeholder="Ask your agents anything..."
              placeholderTextColor={colors.textMuted}
              onSubmitEditing={handleTextSubmit}
              returnKeyType="send"
            />
            <TouchableOpacity style={styles.sendBtn} onPress={handleTextSubmit}>
              <Icon name="send" size={20} color={colors.white} />
            </TouchableOpacity>
          </View>
        )}
      </View>
    </KeyboardAvoidingView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: colors.background,
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingHorizontal: spacing.md,
    paddingTop: spacing.lg,
    paddingBottom: spacing.sm,
  },
  title: {
    color: colors.cyan,
    fontSize: fontSize.xl,
    fontWeight: '800',
    letterSpacing: 1,
  },
  modeToggle: {
    flexDirection: 'row',
    backgroundColor: colors.surface,
    borderRadius: radius.full,
    padding: 2,
  },
  modeBtn: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 4,
    paddingHorizontal: spacing.sm,
    paddingVertical: spacing.xs,
    borderRadius: radius.full,
  },
  modeBtnActive: {
    backgroundColor: colors.surfaceLight,
  },
  modeBtnText: {
    color: colors.textMuted,
    fontSize: fontSize.sm,
    fontWeight: '600',
  },
  modeBtnTextActive: {
    color: colors.white,
  },
  messageArea: {
    flex: 1,
  },
  messageContent: {
    padding: spacing.md,
    gap: spacing.sm,
  },
  emptyState: {
    alignItems: 'center',
    paddingTop: 60,
    gap: spacing.md,
  },
  emptyIcon: {
    width: 80,
    height: 80,
    borderRadius: 40,
    backgroundColor: colors.cyanDim,
    alignItems: 'center',
    justifyContent: 'center',
  },
  emptyTitle: {
    color: colors.text,
    fontSize: fontSize.xl,
    fontWeight: '700',
  },
  emptySubtitle: {
    color: colors.textSecondary,
    fontSize: fontSize.md,
    textAlign: 'center',
    paddingHorizontal: spacing.xl,
    lineHeight: 22,
  },
  suggestions: {
    width: '100%',
    gap: spacing.sm,
    marginTop: spacing.md,
  },
  suggestion: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    backgroundColor: colors.surface,
    padding: spacing.md,
    borderRadius: radius.md,
    borderWidth: 1,
    borderColor: colors.surfaceBorder,
  },
  suggestionText: {
    color: colors.text,
    fontSize: fontSize.md,
  },
  transcriptContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: spacing.sm,
    backgroundColor: colors.surface,
    padding: spacing.md,
    borderRadius: radius.md,
    borderWidth: 1,
    borderColor: colors.cyanDim,
  },
  transcriptDot: {
    width: 8,
    height: 8,
    borderRadius: 4,
    backgroundColor: colors.red,
  },
  transcript: {
    color: colors.text,
    fontSize: fontSize.lg,
    flex: 1,
  },
  messageBubble: {
    padding: spacing.md,
    borderRadius: radius.md,
    maxWidth: '85%',
  },
  userBubble: {
    backgroundColor: colors.cyan,
    alignSelf: 'flex-end',
    borderBottomRightRadius: 4,
  },
  assistantBubble: {
    backgroundColor: colors.surface,
    alignSelf: 'flex-start',
    borderBottomLeftRadius: 4,
    borderWidth: 1,
    borderColor: colors.surfaceBorder,
  },
  assistantIcon: {
    marginBottom: spacing.xs,
  },
  messageText: {
    color: colors.text,
    fontSize: fontSize.md,
    lineHeight: 22,
  },
  userMessageText: {
    color: colors.white,
  },
  progressSection: {
    gap: spacing.sm,
    marginTop: spacing.sm,
  },
  progressTitle: {
    color: colors.textSecondary,
    fontSize: fontSize.sm,
    fontWeight: '600',
  },
  progressGrid: {
    gap: spacing.xs,
  },
  inputArea: {
    paddingHorizontal: spacing.md,
    paddingBottom: spacing.lg,
    paddingTop: spacing.sm,
    borderTopWidth: 1,
    borderTopColor: colors.surfaceBorder,
  },
  voiceArea: {
    alignItems: 'center',
    gap: spacing.sm,
  },
  listeningText: {
    color: colors.red,
    fontSize: fontSize.sm,
    fontWeight: '600',
  },
  textArea: {
    flexDirection: 'row',
    gap: spacing.sm,
    alignItems: 'center',
  },
  textInput: {
    flex: 1,
    backgroundColor: colors.surface,
    borderRadius: radius.full,
    paddingHorizontal: spacing.md,
    paddingVertical: spacing.sm,
    color: colors.text,
    fontSize: fontSize.md,
    borderWidth: 1,
    borderColor: colors.surfaceBorder,
  },
  sendBtn: {
    width: 44,
    height: 44,
    borderRadius: 22,
    backgroundColor: colors.cyan,
    alignItems: 'center',
    justifyContent: 'center',
  },
});
