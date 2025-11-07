port module WebSocket exposing (..)


-- Ports for WebSocket communication


port websocketIn : (String -> msg) -> Sub msg


port websocketOut : String -> Cmd msg


port websocketConnected : (() -> msg) -> Sub msg


port websocketDisconnected : (() -> msg) -> Sub msg
