module Messages.In exposing (IncomingMsgKind(..), kindDecoder)

import Json.Decode exposing (field, string, succeed, fail, Decoder)


type IncomingMsgKind = Default
    | RegisterSuccess
    | Play
    | Pause
    | Resume
    | Stop
    | FsChange
    | FsState

type IncomingMsgPayload = FsStatePayload (List MediaItem)

type alias MediaItem =
  {
    id: Int
  , name: String
  }


-- Payload decoding
payload : IncomingMsgKind -> String -> Maybe IncomingMsgPayload
payload kind json_str =
  case kind of
    FsState -> Result.toMaybe <| Json.Decode.decodeString (Json.Decode.map FsStatePayload <| field "payload" <| Json.Decode.list decodeMediaItem) json_str
    _ -> Nothing     

-- MediaItem decoding
decodeMediaItem : Json.Decode.Decoder MediaItem
decodeMediaItem =
    Json.Decode.map2 MediaItem
        (field "id" Json.Decode.int)
        (field "name" Json.Decode.string)

-- MsgKind decoding
kindDecoder : Decoder IncomingMsgKind
kindDecoder = field "kind" (Json.Decode.string |> Json.Decode.andThen kindDecoderFromString)

kindDecoderFromString : String -> Decoder IncomingMsgKind
kindDecoderFromString string =
  case kindFromString string of
    Just kind -> succeed kind
    Nothing -> fail <| "Can't decode msg kind"

kindFromString : String -> Maybe IncomingMsgKind
kindFromString string =
  case string of
    "RegisterSuccess" -> Just RegisterSuccess
    "Play" -> Just Play
    "Pause" -> Just Pause
    "Resume" -> Just Resume
    "Stop" -> Just Stop
    "FsState" -> Just FsState
    "FsChange" -> Just FsChange
    _ -> Nothing