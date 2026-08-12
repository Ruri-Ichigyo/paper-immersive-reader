import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
} from "react";
import type { ReactNode } from "react";
import type {
  AiRecord,
  Annotation,
  Block,
  Paper,
  ReadingProgress,
  ReadingStats,
  Settings,
  VocabularyItem,
} from "../types";
import { DEFAULT_SETTINGS } from "../types";
import * as db from "../db";
import { GatewayClient } from "../api/gateway";

interface AppContextValue {
  papers: Paper[];
  settings: Settings;
  gatewayClient: GatewayClient;
  refresh: () => Promise<void>;
  getPaper: (paperId: string) => Promise<Paper | undefined>;
  getBlocks: (paperId: string) => Promise<Block[]>;
  getProgress: (paperId: string) => Promise<ReadingProgress | undefined>;
  getAnnotations: (paperId: string) => Promise<Annotation[]>;
  getReadingStats: (paperId: string) => Promise<ReadingStats | undefined>;
  savePaper: (paper: Paper, createdAt: number, updatedAt: number) => Promise<void>;
  saveBlocks: (blocks: Block[]) => Promise<void>;
  saveProgress: (progress: ReadingProgress) => Promise<void>;
  saveAnnotation: (ann: Annotation) => Promise<void>;
  deleteAnnotation: (id: string) => Promise<void>;
  updateAnnotation: (ann: Annotation) => Promise<void>;
  upsertVocabulary: (item: VocabularyItem) => Promise<void>;
  updateVocabulary: (item: VocabularyItem) => Promise<void>;
  deleteVocabulary: (id: string) => Promise<void>;
  saveAiRecord: (rec: AiRecord) => Promise<void>;
  saveSettings: (settings: Settings) => Promise<void>;
  saveReadingStats: (stats: ReadingStats) => Promise<void>;
  deletePaper: (id: string) => Promise<void>;
}

const AppContext = createContext<AppContextValue | null>(null);

export function AppProvider({ children }: { children: ReactNode }) {
  const [papers, setPapers] = useState<Paper[]>([]);
  const [settings, setSettings] = useState<Settings>(DEFAULT_SETTINGS);

  const refresh = useCallback(async () => {
    const stored = await db.listPapers();
    setPapers(stored.map((s) => s.paper).filter(Boolean));
  }, []);

  useEffect(() => {
    (async () => {
      await refresh();
      const loaded = await db.loadSettings();
      if (loaded) setSettings(loaded);
    })();
  }, [refresh]);

  const gatewayBaseUrl = settings.ai_gateway_url ?? "http://127.0.0.1:3000";
  const gatewayClient = useMemo(
    () => new GatewayClient(gatewayBaseUrl, settings.ai_gateway_token_ref ?? null),
    [gatewayBaseUrl, settings.ai_gateway_token_ref],
  );

  const value = useMemo<AppContextValue>(
    () => ({
      papers,
      settings,
      gatewayClient,
      refresh,
      getPaper: db.getPaper,
      getBlocks: db.loadBlocks,
      getProgress: db.loadProgress,
      getAnnotations: db.loadAnnotations,
      getReadingStats: db.loadReadingStats,
      savePaper: db.savePaper,
      saveBlocks: db.saveBlocks,
      saveProgress: db.saveProgress,
      saveAnnotation: db.saveAnnotation,
      deleteAnnotation: db.deleteAnnotation,
      updateAnnotation: db.updateAnnotation,
      upsertVocabulary: db.upsertVocabulary,
      updateVocabulary: db.updateVocabulary,
      deleteVocabulary: db.deleteVocabulary,
      saveAiRecord: db.saveAiRecord,
      saveSettings: async (s) => {
        await db.saveSettings(s);
        setSettings(s);
      },
      saveReadingStats: db.saveReadingStats,
      deletePaper: async (id) => {
        await db.deletePaper(id);
        await refresh();
      },
    }),
    [papers, settings, gatewayClient, refresh],
  );

  return <AppContext.Provider value={value}>{children}</AppContext.Provider>;
}

export function useApp(): AppContextValue {
  const ctx = useContext(AppContext);
  if (!ctx) throw new Error("useApp must be used within AppProvider");
  return ctx;
}