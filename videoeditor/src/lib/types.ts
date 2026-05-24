export type TransitionType =
  | 'cut' | 'fade' | 'crossfade' | 'dip-black' | 'dip-white' | 'slide';

export interface TransitionSpec {
  type: TransitionType;
  duration_ms: number;
}

export interface Resolution {
  width: number;
  height: number;
}

export interface OutputSettings {
  resolution: Resolution;
  framerate: number;
  audio_sample_rate: number;
}

export interface Probe {
  duration_ms: number;
  width: number;
  height: number;
  fps: number;
  video_codec: string;
  audio_codec: string | null;
  has_audio: boolean;
}

export type ProxyStatus = 'pending' | 'generating' | 'ready' | 'failed';

export interface MediaItem {
  id: string;
  source_path: string;
  proxy_path: string | null;
  proxy_status: ProxyStatus;
  probe: Probe | null;
}

export interface VideoClip {
  id: string;
  media_id: string;
  source_in_ms: number;
  source_out_ms: number;
  timeline_start_ms: number;
  volume: number;
  muted: boolean;
  transition_in: TransitionSpec;
  transition_out: TransitionSpec;
}

export interface AudioClip {
  id: string;
  media_id: string;
  source_in_ms: number;
  source_out_ms: number;
  timeline_start_ms: number;
  volume: number;
  fade_in_ms: number;
  fade_out_ms: number;
}

export type TextAnchor =
  | 'tl' | 'tc' | 'tr' | 'ml' | 'mc' | 'mr' | 'bl' | 'bc' | 'br';

export interface TextStyle {
  font_family: string;
  size_px: number;
  color: string;
  weight: number;
  bg_color?: string;
  bg_opacity?: number;
}

export interface TextPosition {
  x_pct: number;
  y_pct: number;
  anchor: TextAnchor;
}

export type TextKind = 'title' | 'caption';

export interface TextClip {
  id: string;
  text: string;
  timeline_start_ms: number;
  duration_ms: number;
  style: TextStyle;
  position: TextPosition;
  kind: TextKind;
}

export interface Timeline {
  duration_ms: number;
  video_track: VideoClip[];
  audio_track: AudioClip[];
  text_track: TextClip[];
}

export interface Project {
  version: string;
  name: string;
  created_at: string;
  modified_at: string;
  output_settings: OutputSettings;
  media_pool: MediaItem[];
  timeline: Timeline;
}

export interface RecentProject {
  path: string;
  name: string;
  last_opened: string;
}

export interface ThumbEntry {
  time_ms: number;
  path: string;
}

export interface Waveform {
  bucket_ms: number;
  peaks: number[];
}
