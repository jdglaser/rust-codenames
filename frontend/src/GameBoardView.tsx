import React from "react";
import CardCell from "./CardCell";
import { Board } from "./Room";

export default function GameBoardView(props: { board: Board, style?: React.CSSProperties }) {
  const { board, style } = props;

  return (
    <div style={style}>
      {board === null ? null : (
        <div style={{ display: "grid", gridTemplateColumns: "repeat(5, 1fr)", gap: "5px" }}>
          {board.map(row => row.map(card => (
            <CardCell key={card.coord.toString()} card={card} />
          )))}
        </div>
      )}
    </div>
  );
}