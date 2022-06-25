
// const uri = ((window.location.protocol === "https:") ? "wss://" : "ws://") + window.location.host + "/ws/" + (room ?? "main");
// const socket = new WebSocket(uri);

import { useEffect, useRef, useState } from "react";
import { useParams } from "react-router-dom";
import { v4 } from "uuid";

enum CardType {
  RED = "RED",
  BLUE = "BLUE",
  BYSTANDER = "BYSTANDER",
  ASSASSIN = "ASSASSIN"
}

type Card = {word: string, cardType: CardType, flipped: boolean}

type Board = Card[][]

type Game = {sessionts: number[], board: Board}

enum EventType {
  Connect = "connect",
  Disconnect = "disconnect",
  TimedOut = "timedOut",
  Message = "message"
}

interface ConnectEvent {
  type: EventType.Connect, 
  data: {id: number, game: Game}
}

interface DisconnectEvent {
  type: EventType.Disconnect,
  data: {id: number}
}

interface TimedOutEvent {
  type: EventType.TimedOut
  data: {id: number}
}

interface ChatMessageEvent {
  type: EventType.Message
  data: {senderId: number, text: string}
}

type Event = ConnectEvent | DisconnectEvent | TimedOutEvent | ChatMessageEvent

function resolveCardType(cardType: CardType): string {
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

export default function Room() {
  const { room } = useParams();

  const [message, setMessage] = useState<string>("");
  const [messages, setMessages] = useState<string[]>([]);

  const [board, setBoard] = useState<Board | null>(null);

  const webSocket = useRef<WebSocket | null>(null);
  const messagesEndRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    if (webSocket.current) {
      console.log("Websocket already setup skipping...")
      return;
    }
    
    const uri = ((window.location.protocol === "https:") ? "wss://" : "ws://") + window.location.host + "/ws/" + (room ?? "main");
    webSocket.current = new WebSocket(uri);

    webSocket.current.onopen = () => {
      console.log("WEBSOCKET OPEN");
    }

    webSocket.current.onclose = () => {
      console.log("WEBSOCKET CLOSED");
    }
    
    webSocket.current.onmessage = (msg: MessageEvent<string>) => {
      console.log("MESSAGE: ", msg.data);
      const event: Event = JSON.parse(msg.data);
      console.log(event);
      const {data} = event;
      switch (event.type) {
        case EventType.Connect:
          setMessages(prev => [...prev, `Got a connect message with data: ${event.data.id}`]);
          setBoard(event.data.game.board)
          break;
        case EventType.Disconnect:
          setMessages(prev => [...prev, `User id ${event.data.id} disconnected from the game!`]);
          break;
        case EventType.TimedOut:
          setMessages(prev => [...prev, `User id ${event.data.id} timed out and has been disconnected from the game!`])
          break;
        case EventType.Message:
          setMessages(prev => [...prev, `${event.data.senderId}: ${event.data.text}`])
          break;
        default:
          console.error("Unrecognized event: ", event);
      }
    };

    return () => {
      console.log("CLOSING")
      if (webSocket.current) {
        webSocket.current.close()
      }
      webSocket.current = null;
    }
  }, []);

  function scrollToBottom() {
    if (messagesEndRef.current === null) {
      return;
    }
    messagesEndRef.current.scrollIntoView({ behavior: "smooth" })
  }

  useEffect(scrollToBottom, [messages]);

  function sendMessage() {
    webSocket.current?.send(JSON.stringify(
      {
        type: "message",
        data: {text: message}
      }
    ))
    setMessage("");
  }

  return (
    <>
      <h1>Hi! Welcome to {room}</h1>
      <input type="text"
                  onChange={(evt) => {
                      evt.preventDefault();
                      setMessage(evt.target.value)
                  }}
                  value={message} />
      <div className='buttons'>
        <button onClick={() => sendMessage()} disabled={message === ""}>Send</button>
      </div>
      {board === null ? null : (
        <div style={{display: "grid", gridTemplateColumns: "auto auto auto auto auto", border: "1px solid green"}}>
          {board.map(row => row.map(card => (
            <div key={v4()} style={{border: "1px solid red", backgroundColor: resolveCardType(card.cardType as CardType)}}>
              {card.word}
            </div>
          )))}
        </div>
      )}
      <div className="messages" style={{border: "1px solid black", maxHeight: "50vh", overflow: "scroll"}}>
        {messages.map(msg => (
          <div key={v4()}>{msg}</div>
        ))}
        <div ref={messagesEndRef} />
      </div>
    </>
  )
}