import React from "react";
import { Card, CardType } from "./Room";

function resolveCardType(card: Card): string {
  const {cardType, flipped} = card;

  if (flipped === false) {
    return "white";
  }

  if (cardType === CardType.BLUE) {
    return "blue";
  }

  if (cardType === CardType.RED) {
    return "red"
  }

  if (cardType === CardType.ASSASSIN) {
    return "grey"
  }

  return "tan"
}

export default function CardCell(props: { card: Card, onFlip: (coord: [number, number]) => void }) {
  const { card, onFlip } = props;



  return (
    <div style={{
          backgroundColor: resolveCardType(card),
          color: (card.flipped && ["BLUE", "RED"].includes(card.cardType)) ? "white" : "",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          padding: "12px 2px",
          borderRadius: "5px",
          border: "1px solid black",
          fontSize: "0.5rem",
          cursor: card.flipped ? "" : "pointer"
        }}
        role="button"
        tabIndex={0}
        onClick={() => onFlip(card.coord)}>
      {card.word}
    </div>
  )
}