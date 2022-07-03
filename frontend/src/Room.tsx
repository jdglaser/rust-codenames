
// const uri = ((window.location.protocol === "https:") ? "wss://" : "ws://") + window.location.host + "/ws/" + (room ?? "main");
// const socket = new WebSocket(uri);

import { useEffect, useRef, useState } from "react";
import { useParams } from "react-router-dom";
import ChatView from "./ChatView";
import GameBoardView from "./GameBoardView";

export enum CardType {
  RED = "RED",
  BLUE = "BLUE",
  BYSTANDER = "BYSTANDER",
  ASSASSIN = "ASSASSIN"
}

export type Card = {word: string, cardType: CardType, flipped: boolean, coord: [number, number]}

export type Board = Card[][]

export type Game = {board: Board, turnTeam: Team, startingTeam: Team}

export type ClientSession = {id: number, username: string, room: string, is_spymaster: boolean}

enum EventType {
  Connect = "connect",
  Disconnect = "disconnect",
  TimedOut = "timedOut",
  Message = "message",
  GameStateUpdate = "gameStateUpdate",
  NewGame = "newGame",
  SetName = "setName"
}

enum Team {
  RED = "RED",
  BLUE = "BLUE"
}

interface ConnectEvent {
  type: EventType.Connect, 
  data: {id: number}
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
  data: {sender: ClientSession, text: string}
}

interface GameStateUpdateEvent {
  type: EventType.GameStateUpdate
  data: {game: Game}
}

interface NewGameEvent {
  type: EventType.NewGame
  data: {}
}

interface SetNameEvent {
  type: EventType.SetName,
  data: {id: number, name: string}
}

type Event = ConnectEvent | DisconnectEvent | TimedOutEvent | ChatMessageEvent | GameStateUpdateEvent | NewGameEvent | SetNameEvent


export default function Room() {
  const { room } = useParams();

  const [message, setMessage] = useState<string>("");
  const [messages, setMessages] = useState<string[]>([]);

  const [game, setGame] = useState<Game | null>(null);
  const [usernameSet, setUsernameSet] = useState<Boolean>(false);
  const [username, setUsername] = useState<string>("");

  const webSocket = useRef<WebSocket | null>(null);

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
      const event: Event = JSON.parse(msg.data);
      switch (event.type) {
        case EventType.Connect:
          setMessages(prev => [...prev, `Got a connect message with data: ${event.data.id}`]);
          break;
        case EventType.Disconnect:
          setMessages(prev => [...prev, `User id ${event.data.id} disconnected from the game!`]);
          break;
        case EventType.TimedOut:
          setMessages(prev => [...prev, `User id ${event.data.id} timed out and has been disconnected from the game!`])
          break;
        case EventType.Message:
          setMessages(prev => [...prev, `${event.data.sender.username}: ${event.data.text}`])
          break;
        case EventType.GameStateUpdate:
          setGame(event.data.game)
          break;
        case EventType.NewGame:
          setMessages(prev => [...prev, "Game restarted!"]);
          break;
        case EventType.SetName:
          setMessages(prev => [...prev, `${event.data.name} joined the game!`]);
          break;
        default:
          console.error("Unrecognized event: ", event);
      }
    };

    return () => {
      if (webSocket.current) {
        webSocket.current.close()
      }
      webSocket.current = null;
    }
  }, []);

  function sendMessage() {
    webSocket.current?.send(JSON.stringify(
      {
        type: "message",
        data: {text: message}
      }
    ));
    setMessage("");
  }

  function restartGame() {
    webSocket.current?.send(JSON.stringify(
      {
        type: "newGame",
        data: {}
      }
    ))
  }

  function onFlip(coord: [number, number]) {
    webSocket.current?.send(JSON.stringify(
      {
        type: "flipCard",
        data: {coord}
      }
    ))
  }

  function onSetUsername() {
    webSocket.current?.send(JSON.stringify(
      {
        type: "setName",
        data: {name: username}
      }
    ));
    setUsernameSet(true);
  }

  if (game === null) {
    return (
      <>
        Loading...
      </>
    )
  }

  if (!usernameSet) {
    return (
      <div>
        Set username
        <input type="text"
               value={username}
               onChange={(evt) => {evt.preventDefault(); setUsername(evt.target.value)}} />
        <button disabled={username === ""} onClick={onSetUsername}>Submit</button>
      </div>
    )
  }

  return (
    <div style={{height: "100%", 
                 width: "100%",
                 maxWidth: "100%",
                 maxHeight: "100%",
                 display: "flex", 
                 flexDirection: "column",
                 padding: "30px",
                 gap: "10px",
                 alignItems: "center"}}>
      <div style={{display: "flex", 
                   flexDirection: "column", 
                   gap: "10px",
                   height: "100%",
                   justifyContent: "center"}}>
        <div style={{display: "flex", flexDirection: "column", gap: "5px", maxWidth: "100%"}}>
          <h2>Welcome to game {room}</h2>
          <div style={{display: "flex", flexDirection: "column", justifyContent: "center", gap: "10px"}}>
            <div style={{color: game.turnTeam === Team.BLUE ? "blue" : "red"}}>
              {game.turnTeam}'s turn!
            </div>
            <button onClick={restartGame}>Restart</button>
          </div>
        </div>
        <GameBoardView style={{width: "100%", 
                               alignSelf: "left"}} 
                       board={game.board}
                       onFlip={onFlip} />
        <ChatView style={{overflow: "scroll", 
                          flexGrow: "1", 
                          fontSize: "0.75rem", 
                          lineHeight: "1.25rem",
                          maxHeight: "200px"}} 
                  chatMessages={messages} /> 
        <div style={{display: "flex", gap: "10px"}}>
          <input type="text"
                onChange={(evt) => {
                  evt.preventDefault();
                  setMessage(evt.target.value);
                }}
                value={message} />
          <button onClick={sendMessage}>Send</button>
        </div>
        </div>
    </div> 
  )
}