import React from "react";
import { Card, CardType } from "./Room";

function resolveCardType(card: Card): string {
  const {cardType, flipped} = card;

  console.log("CARD: ", card)
  console.log("FLIPPED:", flipped)

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

export default function CardCell(props: { card: Card }) {
  const { card } = props;

  return (
    <div style={{
          backgroundColor: resolveCardType(card),
          color: (card.flipped && ["BLUE", "RED"].includes(card.cardType)) ? "white" : "",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          padding: "10px 0px",
          borderRadius: "5px",
          border: "1px solid black",
          fontSize: "0.5rem"
        }}>
      {card.word}
    </div>
  )
}