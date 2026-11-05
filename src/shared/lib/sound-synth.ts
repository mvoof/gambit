/**
 * Every sound is synthesised here: no files, so no sample licences. A sound is
 * a short list of notes through a decaying gain, wood-like and quiet.
 */
import type { MoveSound } from '@shared/contracts/generated/bindings';

export type GameSound = MoveSound | 'gameEnd';

interface Note {
  frequency: number;
  startSeconds: number;
  durationSeconds: number;
  gain: number;
  wave: OscillatorType;
}

const note = (
  frequency: number,
  startSeconds: number,
  durationSeconds: number,
  gain: number,
  wave: OscillatorType = 'triangle'
): Note => ({ frequency, startSeconds, durationSeconds, gain, wave });

const SOUND_NOTES: Record<GameSound, Note[]> = {
  move: [note(190, 0, 0.09, 0.9)],
  capture: [note(150, 0, 0.1, 1), note(95, 0.03, 0.14, 0.8, 'square')],
  castle: [note(190, 0, 0.08, 0.8), note(170, 0.11, 0.09, 0.9)],
  check: [note(660, 0, 0.12, 0.5), note(880, 0.09, 0.2, 0.45)],
  promote: [
    note(392, 0, 0.1, 0.5),
    note(523, 0.09, 0.1, 0.5),
    note(659, 0.18, 0.22, 0.5),
  ],
  gameEnd: [
    note(523, 0, 0.18, 0.45),
    note(392, 0.16, 0.18, 0.45),
    note(261, 0.32, 0.4, 0.5),
  ],
};

const PERCENT = 100;
const SILENCE_GAIN = 0.0001;
const MASTER_GAIN = 0.35;

let audioContext: AudioContext | null = null;

const contextForPlaying = () => {
  audioContext ??= new AudioContext();

  return audioContext;
};

const playNote = (context: AudioContext, played: Note, volume: number) => {
  const oscillator = context.createOscillator();
  const envelope = context.createGain();
  const start = context.currentTime + played.startSeconds;
  const end = start + played.durationSeconds;

  oscillator.type = played.wave;
  oscillator.frequency.value = played.frequency;
  envelope.gain.setValueAtTime(played.gain * volume, start);
  envelope.gain.exponentialRampToValueAtTime(SILENCE_GAIN, end);

  oscillator.connect(envelope).connect(context.destination);
  oscillator.start(start);
  oscillator.stop(end);
};

export const playSound = (sound: GameSound, volumePercent: number) => {
  if (volumePercent <= 0) {
    return;
  }

  const context = contextForPlaying();
  const volume = (volumePercent / PERCENT) * MASTER_GAIN;

  for (const played of SOUND_NOTES[sound]) {
    playNote(context, played, volume);
  }
};
