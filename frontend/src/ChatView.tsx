import React, { ReactElement, useEffect, useRef } from "react";
import { v4 } from "uuid";

export default function ChatView(props: { chatMessages: (string | ReactElement)[], style?: React.CSSProperties }) {
  const { chatMessages, style } = props;

  const messagesEndRef = useRef<HTMLDivElement | null>(null);

  useEffect(scrollToBottom, [chatMessages]);

  function scrollToBottom() {
    if (messagesEndRef.current === null) {
      return;
    }
    messagesEndRef.current.scrollIntoView({ behavior: "smooth" })
  }

  return (
    <div style={style} className="messages">
      {chatMessages.map(msg => (
        <div key={v4()}>{msg}</div>
      ))}
      <div ref={messagesEndRef} />
    </div>
  )
}