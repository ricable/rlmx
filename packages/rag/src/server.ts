import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import { CallToolRequestSchema, ListToolsRequestSchema } from '@modelcontextprotocol/sdk/types.js';
import { createAllRagTools, type RagToolContext } from './tools.js';
import { loadConfig } from './config.js';
import { BackendRegistry } from './backends/backend.js';
import { QmdBackend } from './backends/qmd.js';
import { DoclingBackend } from './backends/docling.js';
import { MemoryBackend } from './backends/memory.js';
import { ProvenanceTracker } from './provenance.js';

export class RagServer {
  private server: Server;
  private ctx: RagToolContext;

  constructor(overrides?: Partial<import('./types.js').RagConfig>) {
    const config = loadConfig(overrides);
    const registry = new BackendRegistry();

    registry.register(new QmdBackend(config.qmdUrl, config.qmdCollection));
    if (config.doclingEnabled) {
      registry.register(new DoclingBackend(config.ingestedDir));
    }
    registry.register(new MemoryBackend());

    this.ctx = { config, registry, provenance: new ProvenanceTracker() };

    this.server = new Server(
      { name: '@aix/rag', version: '0.1.0' },
      { capabilities: { tools: {} } },
    );

    this.registerHandlers();
  }

  private registerHandlers(): void {
    const tools = createAllRagTools();

    this.server.setRequestHandler(ListToolsRequestSchema, async () => ({
      tools: tools.map(t => ({
        name: t.definition.name,
        description: t.definition.description,
        inputSchema: t.definition.inputSchema,
      })),
    }));

    this.server.setRequestHandler(CallToolRequestSchema, async (request) => {
      const tool = tools.find(t => t.definition.name === request.params.name);
      if (!tool) {
        return {
          content: [{ type: 'text' as const, text: `Unknown tool: ${request.params.name}` }],
          isError: true,
        };
      }

      try {
        const result = await tool.handler(
          (request.params.arguments ?? {}) as Record<string, unknown>,
          this.ctx,
        );
        return {
          content: [{ type: 'text' as const, text: JSON.stringify(result, null, 2) }],
        };
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        return {
          content: [{ type: 'text' as const, text: message }],
          isError: true,
        };
      }
    });
  }

  async start(): Promise<void> {
    const transport = new StdioServerTransport();
    await this.server.connect(transport);
  }
}
