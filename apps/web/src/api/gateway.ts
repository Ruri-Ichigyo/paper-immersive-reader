// Client for the Rust gateway (Axum) HTTP API.

export interface TranslateResponse {
  translation: string;
  source_text: string;
  target_language: string;
  normalized_term: string | null;
}

export interface AiResult {
  record_type: string;
  input: string;
  output: string;
  language: string;
  normalized_term?: string | null;
}

export interface ExplainResponse {
  explanation: string;
  background: string | null;
  caution: string | null;
}

export class GatewayClient {
  private baseUrl: string;
  private token: string | null;

  constructor(baseUrl: string, token: string | null) {
    this.baseUrl = baseUrl.replace(/\/+$/, "");
    this.token = token;
  }

  private headers(): Record<string, string> {
    const h: Record<string, string> = { "Content-Type": "application/json" };
    if (this.token) {
      h["Authorization"] = `Bearer ${this.token}`;
    }
    return h;
  }

  private async post<T>(path: string, body: unknown): Promise<T> {
    const resp = await fetch(`${this.baseUrl}${path}`, {
      method: "POST",
      headers: this.headers(),
      body: JSON.stringify(body),
    });
    if (resp.status === 401) {
      throw new Error("unauthorized — check gateway token in Settings");
    }
    if (!resp.ok) {
      const text = await resp.text();
      throw new Error(`gateway error ${resp.status}: ${text}`);
    }
    return (await resp.json()) as T;
  }

  async health(): Promise<boolean> {
    try {
      const resp = await fetch(`${this.baseUrl}/health`);
      return resp.ok;
    } catch {
      return false;
    }
  }

  async translate(text: string, targetLanguage: string, context?: string): Promise<string> {
    const resp = await this.post<TranslateResponse>("/api/translate", {
      text,
      target_language: targetLanguage,
      context: context ?? null,
    });
    return resp.translation;
  }

  async explain(
    text: string,
    explanationType: string,
    language: string,
    context?: string,
  ): Promise<AiResult> {
    const resp = await this.post<ExplainResponse>("/api/explain", {
      text,
      explanation_type: explanationType,
      language,
      context: context ?? null,
    });
    let output = resp.explanation;
    if (resp.background) {
      output += `\n\nBackground: ${resp.background}`;
    }
    if (resp.caution) {
      output += `\n\nNote: ${resp.caution}`;
    }
    return {
      record_type: `explain_${explanationType}`,
      input: text,
      output,
      language,
      normalized_term: null,
    };
  }
}