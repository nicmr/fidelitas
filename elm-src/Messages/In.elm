module Messages.In exposing (IncomingMsgKind(..), kindFieldDecoder)

import Json.Decode exposing (field, string, succeed, fail, Decoder)


type IncomingMsgKind = FsState
    | Default
    | RegisterSuccess
    | Play
    | Pause
    | Resume
    | Stop
    | FsChange

kindFieldDecoder: Decoder IncomingMsgKind
kindFieldDecoder = field "kind" kindDecoder

kindDecoder: Decoder IncomingMsgKind
kindDecoder = Json.Decode.string |> Json.Decode.andThen kindFromString

kindFromString: String -> Decoder IncomingMsgKind
kindFromString string =
  case string of
    "RegisterSuccess" -> succeed RegisterSuccess
    "Play" -> succeed Play
    "Pause" -> succeed Pause
    "Resume" -> succeed Resume
    "Stop" -> succeed Stop
    "FsState" -> succeed FsState
    "FsChange" -> succeed FsChange
    _ -> fail <| "Cannot decode"