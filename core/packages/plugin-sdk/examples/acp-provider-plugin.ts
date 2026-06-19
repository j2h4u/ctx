import { acpProvider, defineCtxPlugin, entrypoint } from "../src/index.js";

export default defineCtxPlugin({
  id: "example.acp-provider",
  name: "Example ACP Provider",
  version: "0.1.0",
  description: "Example local ACP v1 provider plugin manifest.",
  entrypoints: [
    entrypoint({
      id: "agent",
      kind: "process",
      command: "example-agent-acp",
    }),
  ],
  contributes: {
    providers: [
      acpProvider({
        id: "example-agent",
        name: "Example Agent",
        entrypoint: "agent",
      }),
    ],
  },
  compatibility: {
    capabilities: ["plugins.manifest.v1", "acp.v1.local-stdio"],
  },
});
