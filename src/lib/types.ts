export interface VideoFormat {
  id: string;
  ext: string;
  resolution?: string;
  fps?: number;
  vcodec?: string;
  acodec?: string;
  filesize?: number;
  tbr?: number;
  note?: string;
}

export interface VideoMeta {
  url: string;
  title: string;
  thumbnail?: string;
  duration?: number;
  uploader?: string;
  formats: VideoFormat[];
  is_playlist?: boolean;
  playlist_count?: number;
}

export interface DownloadJob {
  id: string;
  url: string;
  title: string;
  format_id: string;
  output_dir: string;
  thumbnail?: string;
}

export type JobStatus =
  | { kind: "queued" }
  | { kind: "running"; downloaded: number; total: number; speed: number; eta: number }
  | { kind: "done"; path: string }
  | { kind: "error"; message: string }
  | { kind: "cancelled" };

export interface JobState extends DownloadJob {
  status: JobStatus;
  started_at: number;
  ended_at?: number;
}

export interface UpdatePolicy {
  min_required_version: string;
  message: string;
}
