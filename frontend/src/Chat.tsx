
// const uri = ((window.location.protocol === "https:") ? "wss://" : "ws://") + window.location.host + "/ws/" + (room ?? "main");
// const socket = new WebSocket(uri);

import { useEffect, useRef, useState } from "react";
import { useParams } from "react-router-dom";

export default function Chat() {
  const { room } = useParams();

  const [message, setMessage] = useState<string>("");
  const [messages, setMessages] = useState<string[]>([]);

  const webSocket = useRef<WebSocket | null>(null);
  const messagesEndRef = useRef<HTMLElement | null>(null);

  useEffect(() => {
    const uri = ((window.location.protocol === "https:") ? "wss://" : "ws://") + window.location.host + "/ws/" + (room ?? "main");
    webSocket.current = new WebSocket(uri);

    webSocket.current.onopen = () => {
      console.log("WEBSOCKET OPEN");
    }
    
    webSocket.current.onmessage = (msg) => {
      console.log("MESSAGE: ", msg.data);
      setMessages(prev => [...prev, msg.data]);
    };

    return () => webSocket.current?.close()
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
        type: "Message",
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
      <div className="messages" style={{border: "1px solid black", maxHeight: "50vh", overflow: "scroll"}}>
        {messages.map(msg => (
          <div key={msg}>{msg}</div>
        ))}
        <div ref={messagesEndRef} />
      </div>
    </>
  )
}