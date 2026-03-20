#!/usr/bin/env node
import { RagServer } from './server.js';

const server = new RagServer();
server.start().catch((err) => {
  process.stderr.write(`@aix/rag server failed: ${err}\n`);
  process.exit(1);
});
