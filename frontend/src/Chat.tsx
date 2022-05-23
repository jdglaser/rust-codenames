import { useEffect, useState } from "react";
import { useLocation, useParams } from "react-router-dom";

export default function Chat() {
  const [ws, setWs] = useState<WebSocket>();
  const [msg, setMsg] = useState<string>("");

  const {room} = useParams();

  const location = useLocation();

  const state = location.state as Record<string, string>;

  useEffect(() => {
    console.log("Setting up websocket connection");
    const uri = ((window.location.protocol === "https:") ? "wss://" : "ws://") + window.location.host + "/ws/" + (room ?? "main");
    let socket = new WebSocket(uri);
    console.log(uri);

    socket.onopen = () => {
     console.log("WS OPEN") 
    }

    // message received - show the message in div#messages
    socket.onmessage = function(event) {
      let message = event.data;

      console.log("MESSAGE RECIEVED: ", message);
    }

    setWs(socket);
  }, [])

  return (
    <>
      <h1>Chat {state.userName}</h1>
      <input type="text"
             onChange={(evt) => {
               evt.preventDefault();
               setMsg(evt.target.value);
              }}
              value={msg} />
      <button onClick={() => ws?.send(msg)}>Submit</button>
    </>
  )
}