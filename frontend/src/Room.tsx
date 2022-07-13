import { KeyboardEvent, ReactElement, useEffect, useRef, useState } from "react";
import { useCookies } from "react-cookie";
import { useMediaQuery } from "react-responsive";
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

enum Team {
  RED = "RED",
  BLUE = "BLUE"
}

enum GameStatusType {
  PLAYING = "PLAYING",
  OVER = "OVER"
}

interface PlayingGameStatus {
  type: GameStatusType.PLAYING
  data: {}
}

interface OverGameStatus {
  type: GameStatusType.OVER
  data: {winner: Team}
}

type GameStatus = PlayingGameStatus | OverGameStatus

export type Game = {board: Board, turnTeam: Team, startingTeam: Team, remainingCards: [number, number], gameStatus: GameStatus}

export type ClientSession = {id: number, username: string, room: string, is_spymaster: boolean}

enum EventType {
  Connect = "connect",
  Disconnect = "disconnect",
  TimedOut = "timedOut",
  Message = "message",
  GameStateUpdate = "gameStateUpdate",
  NewGame = "newGame",
  SetName = "setName",
  FlipCard = "flipCard",
  UpdateClientSession = "updateClientSession",
  SetSpyMaster = "setSpyMaster",
  NextTurn = "nextTurn"
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

interface UpdateClientSessionEvent {
  type: EventType.UpdateClientSession
  data: {session: ClientSession}
}

interface SetSpyMasterEvent {
  type: EventType.SetSpyMaster,
  data: {}
}

interface NextTurnEvent {
  type: EventType.NextTurn
  data: {}
}

type Event = ConnectEvent | DisconnectEvent | TimedOutEvent | ChatMessageEvent | 
  GameStateUpdateEvent | NewGameEvent | SetNameEvent | FlipCardEvent | 
  UpdateClientSessionEvent | SetSpyMasterEvent | NextTurnEvent

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
  const [myClientSession, setMyClientSession] = useState<ClientSession>();

  const webSocket = useRef<WebSocket | null>(null);
  const prevGameState = useRef<Game>();
  const inputRef = useRef<HTMLInputElement>(null);

  const [cookies, setCookie] = useCookies(["username"]);

  const usernameIsSet = cookies.username !== undefined;

  const gameOver = game ? game.gameStatus.type == GameStatusType.OVER : false;
  const isSpymaster = myClientSession ? myClientSession.is_spymaster : false;
  // const showCards = gameOver || (myClientSession ? myClientSession.is_spymaster : false)

  const isLandscape = useMediaQuery({query: "(orientation: landscape)"});
  const isDesktop = useMediaQuery({query: "(min-width: 1025px)"});

  useEffect(() => {
    if (game) {
      prevGameState.current = game;
    }
  }, [game]);

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
          if (prevGameState.current?.turnTeam && prevGameState.current?.turnTeam !== event.data.game.turnTeam) {
            const {turnTeam} = event.data.game;
            setMessages(prev => [...prev, (
              <>
                It is now <span style={{color: turnTeam === "BLUE" ? "blue" : "red"}}>{turnTeam}'s</span> turn! 
              </>
            )])
          }
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
              {sender.username} flipped card "{card.word}". The card was <span style={{fontWeight: "bold", color: resolveCardTypeColor(card, true, true)}}>{card.cardType}</span>!
            </>
          )])
          break;
        case EventType.UpdateClientSession:
          console.log("Client session updated")
          setMyClientSession(event.data.session)
          break;
        case EventType.SetSpyMaster:
          setMessages(prev => [...prev, `${sender.username} set themselves as the spymaster!`])
          break;
        case EventType.NextTurn:
          setMessages(prev => [...prev, `${sender.username} advanced the turn.`])
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

  function handleEnterPressed(e: KeyboardEvent<HTMLInputElement>) {
    if (e.key == "Enter") {
      sendMessage()
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
      window.removeEventListener("focus", handleWebSocketFocus);
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

  function setSpymaster() {
    webSocket.current?.send(JSON.stringify(
      {
        type: "setSpyMaster",
        data: {spymaster: true}
      }
    ))
  }

  function nextTurn() {
    webSocket.current?.send(JSON.stringify(
      {
        type: "nextTurn",
        data: {}
      }
    ))
  }

  function onSetUsername() {
    const expireDate = new Date()
    expireDate.setFullYear(expireDate.getFullYear() + 5);
    setCookie("username", username, {path: "/", expires: expireDate});
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
                 display: "flex", 
                 flexDirection: "column",
                 padding: "8px",
                 gap: "10px",
                 alignItems: "center",
                 justifyContent: "center"}}>
      <div style={{display: "flex", 
                   flexDirection: "column",
                   gap: "10px",
                   height: isDesktop ? "90%" : "100%",
                   width: isDesktop ? "90%" : "100%",
                   justifyContent: "center"}}>
        <div style={{display: "flex", flexDirection: "column", gap: "5px", maxWidth: "100%"}}>
          <h2>Welcome to game {room}</h2>
          <div style={{display: "flex", flexDirection: "column", justifyContent: "center", gap: "10px"}}>
            <div style={{display: "flex", flexDirection: "row", gap: "25px", alignItems: "center"}}>
              <div style={{width: "50px", textAlign: "center", display: "grid", gridTemplateColumns: "auto auto", gap: "1px", backgroundColor: "black", border: "1px solid black", borderRadius: "5px"}}>
                <div style={{color: "blue", padding: "5px", backgroundColor: "white", borderRadius: "4px 0 0 4px"}}>{game.remainingCards[0]}</div>
                <div style={{color: "red", padding: "5px", backgroundColor: "white", borderRadius: "0 4px 4px 0"}}>{game.remainingCards[1]}</div>
              </div>
              {game.gameStatus.type == GameStatusType.OVER ? (
                <div>
                  Game over! <span style={{width: "150px", color: game.gameStatus.data.winner === Team.BLUE ? "blue" : "red"}}>{game.gameStatus.data.winner}</span> team wins!
                </div>
              ) : (
                <div style={{width: "150px", color: game.turnTeam === Team.BLUE ? "blue" : "red"}}>
                  {game.turnTeam}'s turn!
                </div>
              )}
            </div>
            <div style={{display: "flex", gap: "10px"}}>
              <button onClick={restartGame}>Restart</button>
              <button onClick={setSpymaster}>Spymaster</button>
              <button onClick={nextTurn}>Next turn</button>
            </div>
          </div>
        </div>
        <div style={{display: "grid", 
                     gridTemplateColumns: isLandscape ? "1fr 1fr" : "1fr", 
                     gridTemplateRows: isLandscape ? "1fr" : "1fr 1fr",
                     gap: "10px",
                     overflow: "hidden", 
                     justifyContent: "center", 
                     alignContent: "center",
                     height: "100%",
                     width: "100%"}}>
          <GameBoardView board={game.board}
                         onFlip={onFlip}
                         gameOver={gameOver}
                         isSpymaster={isSpymaster} />
          <div style={{display: "flex",
                      flexDirection: "column", 
                      height: "100%",
                      justifyContent: "flex-end",
                      overflow: "hidden"}}>
            <ChatView chatMessages={messages}
                      style={{overflow: "scroll",
                              fontSize: "0.75rem", 
                              lineHeight: "1.25rem",
                              maxHeight: "100%",
                              display: "flex",
                              flexDirection: "column"}} /> 
            <div style={{display: "flex", gap: "10px"}}>
              <input type="text"
                    onChange={(evt) => {
                      setMessage(evt.target.value);
                    }}
                    value={message}
                    style={{flex: "0.8"}}
                    ref={inputRef}
                    onKeyDown={(evt) => {
                      handleEnterPressed(evt)
                    }} />
              <button type="submit" style={{flex: "0.2"}} onClick={sendMessage}>Send</button>
            </div>
          </div>
        </div>
        </div>
    </div> 
  )
}