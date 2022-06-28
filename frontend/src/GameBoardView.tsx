import React from "react";
import CardCell from "./CardCell";
import { Board } from "./Room";

export default function GameBoardView(props: { board: Board, onFlip: (coord: [number, number]) => void, style?: React.CSSProperties }) {
  const { board, style, onFlip } = props;

  return (
    <div style={style}>
      {board === null ? null : (
        <div style={{ display: "grid", gridTemplateColumns: "repeat(5, 1fr)", gap: "8px" }}>
          {board.map(row => row.map(card => (
            <CardCell key={card.coord.toString()} card={card} onFlip={onFlip} />
          )))}
        </div>
      )}
    </div>
  );
}