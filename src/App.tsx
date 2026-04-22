import { CharacterStage } from "./components/character-stage";

export function App() {
  return (
    <>
      {/* Always-visible boot indicator so we can tell whether React mounted
          even when the stage is fully transparent and empty. Tiny dot in
          top-right corner. Remove once rendering is stable. */}
      <div
        style={{
          position: "fixed",
          top: 4,
          right: 4,
          width: 10,
          height: 10,
          borderRadius: 5,
          background: "#4ade80",
          boxShadow: "0 0 4px rgba(0,0,0,0.5)",
          zIndex: 9999,
          pointerEvents: "none",
        }}
        aria-label="react-mounted"
        title="React mounted"
      />
      <CharacterStage />
    </>
  );
}
