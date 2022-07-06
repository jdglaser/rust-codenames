import { ReactElement, useEffect, useRef, useState } from "react";
import { useCookies } from "react-cookie";
import { useParams } from "react-router-dom";
import { resolveCardTypeColor } from "./CardCell";
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

export type Game = {board: Board, turnTeam: Team, startingTeam: Team, remainingCards: [number, number]}

export type ClientSession = {id: number, username: string, room: string, is_spymaster: boolean}

enum EventType {
  Connect = "connect",
  Disconnect = "disconnect",
  TimedOut = "timedOut",
  Message = "message",
  GameStateUpdate = "gameStateUpdate",
  NewGame = "newGame",
  SetName = "setName",
  FlipCard = "flipCard"
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

interface FlipCardEvent {
  type: EventType.FlipCard
  data: {flippedCard: Card}
}

type Event = ConnectEvent | DisconnectEvent | TimedOutEvent | ChatMessageEvent | GameStateUpdateEvent | NewGameEvent | SetNameEvent | FlipCardEvent

interface EventMessage {
  sender: ClientSession
  room: string
  event: Event
}

export default function Room() {
  const { room } = useParams();

  const [message, setMessage] = useState<string>("");
  const [messages, setMessages] = useState<(string | ReactElement)[]>([]);

  const [game, setGame] = useState<Game | null>(null);
  const [username, setUsername] = useState<string>("");

  const webSocket = useRef<WebSocket | null>(null);

  const [cookies, setCookie] = useCookies(["username"]);

  const usernameIsSet = cookies.username !== undefined;

  function setupWebSocket() {
    if (webSocket.current && webSocket.current.readyState === webSocket.current.OPEN) {
      console.log("Websocket already setup skipping...")
      return;
    }
    
    const uri = ((window.location.protocol === "https:") ? "wss://" : "ws://") + window.location.host + "/ws/" + (room ?? "main");
    webSocket.current = new WebSocket(uri);

    webSocket.current.onopen = () => {
      console.log("WEBSOCKET OPEN");
      if (usernameIsSet) {
        webSocket.current?.send(JSON.stringify(
          {
            type: "setName",
            data: {name: cookies.username}
          }
        ));
      }
    }

    webSocket.current.onclose = () => {
      console.log("WEBSOCKET CLOSED");
    }
    
    webSocket.current.onmessage = (msg: MessageEvent<string>) => {
      const eventMessage: EventMessage = JSON.parse(msg.data);
      const {event, sender} = eventMessage;
      switch (event.type) {
        case EventType.Connect:
          break;
        case EventType.Disconnect:
          setMessages(prev => [...prev, `${sender.username} disconnected from the game.`]);
          break;
        case EventType.TimedOut:
          setMessages(prev => [...prev, `${sender.username === "" ? sender.id : sender.username} timed out and has been disconnected from the game.`])
          break;
        case EventType.Message:
          setMessages(prev => [...prev, `${sender.username}: ${event.data.text}`])
          break;
        case EventType.GameStateUpdate:
          setGame(event.data.game)
          break;
        case EventType.NewGame:
          setMessages(prev => [...prev, `${sender.username} restarted the game.`]);
          break;
        case EventType.SetName:
          setMessages(prev => [...prev, `${event.data.name} joined the game!`]);
          break;
        case EventType.FlipCard:
          const {flippedCard: card} = event.data
          setMessages(prev => [...prev, (
            <>
              {sender.username} flipped card "{card.word}". The card was <span style={{color: resolveCardTypeColor(card)}}>{card.cardType}</span>!
            </>
          )])
          break;
        default:
          console.error("Unrecognized event: ", event);
      }
    };
  }

  function handleWebSocketFocus() {
    if (!webSocket.current || webSocket.current.readyState === WebSocket.CLOSED || webSocket.current.readyState === WebSocket.CLOSING) {
      console.log("Websocket closed, retrying setup")
      setupWebSocket();
    }
  }

  useEffect(() => {
    setupWebSocket();

    window.addEventListener("focus", handleWebSocketFocus)

    return () => {
      if (webSocket.current) {
        webSocket.current.close()
      }
      webSocket.current = null;
      window.removeEventListener("focus", handleWebSocketFocus)
    }
  }, []);

  useEffect(() => {
    if (usernameIsSet && webSocket.current?.readyState === 1) {
      webSocket.current.send(JSON.stringify(
        {
          type: "setName",
          data: {name: cookies.username}
        }
      ));
    }
  }, [cookies]);

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
    setCookie("username", username, {path: "/"});
  }

  if (game === null) {
    return (
      <div style={{width: "100%", 
                   height: "100%",
                   display: "flex", 
                   justifyContent: "center", 
                   alignItems: "center"}}>
        Loading...
      </div>
    )
  }

  if (!usernameIsSet) {
    return (
      <div style={{width: "100%", 
                   height: "100%",
                   display: "grid", 
                   gap: "10px",
                   gridTemplateColumns: "auto auto",
                   justifyContent: "center", 
                   alignContent: "center"}}>
        <label>Set username</label>
        <div />
        <input type="text"
               value={username}
               onChange={(evt) => {
                 evt.preventDefault(); 
                 setUsername(evt.target.value)
                }} />
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
            <div style={{display: "flex", flexDirection: "row", justifyContent: "space-between"}}>
              <div style={{color: game.turnTeam === Team.BLUE ? "blue" : "red"}}>
                {game.turnTeam}'s turn!
              </div>
              <div>
                <span style={{color: "blue"}}>{game.remainingCards[0]}</span>
                , 
                <span style={{color: "red"}}>{game.remainingCards[1]}</span>
              </div>
            </div>
            <button onClick={restartGame}>Restart</button>
          </div>
        </div>
        <GameBoardView style={{width: "100%", 
                               alignSelf: "left"}} 
                       board={game.board}
                       onFlip={onFlip} />
        <div style={{display: "flex", 
                     flexGrow: "1",
                     flexDirection: "column", 
                     maxHeight: "200px",
                     justifyContent: "flex-end"}}>
          <ChatView style={{overflow: "scroll",
                            fontSize: "0.75rem", 
                            lineHeight: "1.25rem",
                            maxHeight: "100%",
                            display: "flex",
                            flexDirection: "column"}} 
                    chatMessages={messages} /> 
        </div>
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