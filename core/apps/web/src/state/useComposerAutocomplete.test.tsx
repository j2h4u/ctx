import { act, render, screen, fireEvent } from "@testing-library/react";
import { useRef, useState } from "react";
import { describe, expect, it, vi } from "vitest";
import { ComposerAutocompleteMenu } from "../components/ComposerAutocompleteMenu";
import { useComposerAutocomplete, type SlashCommandDescriptor } from "./useComposerAutocomplete";

function TestHarness({
  initial,
  onSend,
  slashCommands = [{ name: "review", description: "Review changes" }],
}: {
  initial: string;
  onSend: () => void;
  slashCommands?: SlashCommandDescriptor[];
}) {
  const [value, setValue] = useState(initial);
  const ref = useRef<HTMLTextAreaElement | null>(null);
  const ac = useComposerAutocomplete({
    sessionId: null,
    workspaceId: null,
    value,
    setValue,
    textareaRef: ref,
    slashCommands,
  });

  return (
    <div>
      <textarea
        ref={ref}
        value={value}
        onChange={(e) => setValue(e.target.value)}
        onKeyDown={(e) => {
          if (ac.onKeyDown(e)) return;
          if (e.key === "Enter") onSend();
        }}
        onClick={() => ac.syncFromDom()}
        onKeyUp={() => ac.syncFromDom()}
        onSelect={() => ac.syncFromDom()}
      />
      <ComposerAutocompleteMenu
        open={ac.open}
        loading={ac.loading}
        items={ac.items}
        activeIndex={ac.activeIndex}
        onPick={ac.pick}
        onHoverIndex={(i) => ac.setActiveIndex(i)}
        anchorRect={ac.anchorRect}
        inlineFallback={true}
      />
    </div>
  );
}

describe("useComposerAutocomplete integration", () => {
  it("accepts with Enter when suggestions are open (does not send)", async () => {
    vi.spyOn(window, "requestAnimationFrame").mockImplementation((cb: FrameRequestCallback) => {
      return window.setTimeout(() => cb(performance.now()), 0) as unknown as number;
    });

    const onSend = vi.fn();
    render(<TestHarness initial={"do /rev"} onSend={onSend} />);

    const textarea = (await screen.findByDisplayValue("do /rev")) as HTMLTextAreaElement;
    textarea.setSelectionRange(textarea.value.length, textarea.value.length);
    await act(async () => {
      fireEvent.click(textarea);
    });
    await act(async () => {
      await new Promise((r) => setTimeout(r, 0));
    });
    await act(async () => {
      fireEvent.keyDown(textarea, { key: "Enter" });
    });
    await act(async () => {
      await new Promise((r) => setTimeout(r, 0));
    });

    expect(onSend).not.toHaveBeenCalled();
    expect((textarea as HTMLTextAreaElement).value).toBe("do /review ");
    vi.restoreAllMocks();
  });

  it("shows source labels for duplicate slash command names and preserves insertion", async () => {
    vi.spyOn(window, "requestAnimationFrame").mockImplementation((cb: FrameRequestCallback) => {
      return window.setTimeout(() => cb(performance.now()), 0) as unknown as number;
    });

    const onSend = vi.fn();
    render(
      <TestHarness
        initial={"do /rev"}
        onSend={onSend}
        slashCommands={[
          {
            name: "review",
            description: "Provider review",
            source: {
              kind: "provider",
              providerId: "codex",
              protocol: "ACP",
              label: "Codex",
            },
          },
          {
            name: "review",
            description: "Plugin review",
            source: {
              kind: "plugin",
              pluginId: "review.tools",
              pluginName: "Review Tools",
              label: "Review Tools",
            },
          },
        ]}
      />,
    );

    const textarea = (await screen.findByDisplayValue("do /rev")) as HTMLTextAreaElement;
    textarea.setSelectionRange(textarea.value.length, textarea.value.length);
    await act(async () => {
      fireEvent.click(textarea);
    });
    await act(async () => {
      await new Promise((r) => setTimeout(r, 0));
    });

    expect(screen.getAllByText("/review")).toHaveLength(2);
    expect(screen.getByText("Codex")).toBeInTheDocument();
    expect(screen.getByText("Review Tools")).toBeInTheDocument();

    await act(async () => {
      fireEvent.keyDown(textarea, { key: "ArrowDown" });
    });
    await act(async () => {
      fireEvent.keyDown(textarea, { key: "Enter" });
    });
    await act(async () => {
      await new Promise((r) => setTimeout(r, 0));
    });

    expect(onSend).not.toHaveBeenCalled();
    expect(textarea.value).toBe("do /review ");
    vi.restoreAllMocks();
  });
});
