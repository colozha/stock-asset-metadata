export interface Metadata {
  title: string;
  description: string;
  keywords: string[];
}
export type MetadataPatch = Partial<Metadata>;
export interface Asset {
  path: string;
  name: string;
  format: string;
  size: number;
  metadata: Metadata;
  error: string | null;
}
export interface AssetDraft extends Asset {
  draft: Metadata;
  lastSavedCopy: Metadata | null;
  savedCopy: string | null;
  saveError: string | null;
  backupPath: string | null;
}
export type SaveMode = "copy" | "replace_with_backup";
export interface SaveResult {
  path: string;
  outputPath: string | null;
  backupPath: string | null;
  metadata: Metadata | null;
  error: string | null;
}
export interface Progress {
  completed: number;
  total: number;
}
