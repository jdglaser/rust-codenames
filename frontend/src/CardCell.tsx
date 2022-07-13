import React from "react";
import { useMediaQuery } from "react-responsive";
import { Card, CardType } from "./Room";

export function resolveCardTypeColor(card: Card, gameOver: boolean, isSpymaster: boolean): string {
  const {cardType, flipped} = card;

  if (!gameOver && !isSpymaster && !flipped) {
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

export default function CardCell(props: { card: Card, onFlip: (coord: [number, number]) => void, gameOver: boolean, isSpymaster: boolean }) {
  const { card, onFlip, gameOver, isSpymaster } = props;

  const isLandscape = useMediaQuery({query: "(orientation: landscape)"});
  const isDesktop = useMediaQuery({query: "(min-width: 1025px)"});

  const resolveFontSize = () => {
    if (isDesktop) {
      return "0.75rem"
    }
    
    if (isLandscape) {
      return "0.6rem"
    }

    else return "0.5rem"
  }

  return (
    <div style={{
          backgroundColor: resolveCardTypeColor(card, gameOver, isSpymaster),
          color: ((gameOver || isSpymaster || card.flipped) && ["BLUE", "RED"].includes(card.cardType)) ? "white" : "",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          padding: "12px 2px",
          borderRadius: "5px",
          border: "1px solid black",
          fontSize: resolveFontSize(),
          cursor: card.flipped || gameOver || isSpymaster ? "" : "pointer",
          opacity: isSpymaster && card.flipped ? "30%" : ""
        }}
        role="button"
        tabIndex={0}
        onClick={() => {
          if (card.flipped || gameOver || isSpymaster) {
            return;
          }
          onFlip(card.coord)
        }}>
      {card.word}
    </div>
  )
}