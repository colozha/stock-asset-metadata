import type { AppError } from "./i18n";
export interface Metadata {
  title: string;
  description: string;
  keywords: string[];
}
export type MetadataPatch = Partial<Metadata>;
export interface ShutterstockCsvResult {
  directory: string;
  outputPath: string | null;
  rowCount: number;
  error: AppError | null;
}
export interface SaveResponse {
  results: SaveResult[];
  csvResults: ShutterstockCsvResult[];
}
export interface Asset {
  path: string;
  name: string;
  format: string;
  size: number;
  createdAt: number | null;
  modifiedAt: number | null;
  metadata: Metadata;
  error: AppError | string | null;
}
export interface AssetDraft extends Asset {
  draft: Metadata;
  additionalContext: string;
  lastSavedCopy: Metadata | null;
  savedCopy: string | null;
  saveError: AppError | string | null;
  backupPath: string | null;
}
export type SaveMode = "copy" | "replace_with_backup";
export interface SaveResult {
  path: string;
  outputPath: string | null;
  backupPath: string | null;
  metadata: Metadata | null;
  error: AppError | string | null;
}
export interface Progress {
  completed: number;
  total: number;
}

export interface ProviderSetting {
  providerName: string;
  endpointUrl: string;
  modelName: string;
  visionOverride: boolean | null;
  isActive: boolean;
}

export interface KeyStatus {
  providerName: string;
  hasApiKey: boolean;
}

export interface AiSettings {
  providerSettings: ProviderSetting[];
  activeProvider: string;
  descriptionLength: "short" | "medium" | "long";
  keywordCount: number;
  keyStatus: KeyStatus[];
  keyringError: AppError | string | null;
}

export interface AiModelOption {
  modelName: string;
  label: string;
  supportsVision: boolean | null;
}

export interface ProviderDraft {
  providerName: string;
  endpointUrl: string;
  modelName: string;
  visionOverride: boolean | null;
  apiKey: string | null;
}

export interface AiGenerationRequest {
  path: string;
  assetName: string;
  additionalContext: string;
  metadata: Metadata;
  descriptionLength: "short" | "medium" | "long";
  keywordCount: number;
}

export interface GeneratedTitleDescription {
  title: string;
  description: string;
  providerName: string;
  modelName: string;
}

export interface GeneratedKeywords {
  keywords: string[];
  providerName: string;
  modelName: string;
}

export type Language = "id" | "en";
export type Theme = "light" | "dark";
export interface AppPreferences {
  language: Language;
  theme: Theme;
  saveMode: SaveMode;
  outputDir: string;
  folders: string[];
}
export interface FolderStatus {
  path: string;
  error: AppError | string | null;
}
export interface Inspection {
  assets: Asset[];
  folders: FolderStatus[];
  files: string[];
}
