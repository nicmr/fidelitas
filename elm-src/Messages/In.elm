module Messages.In exposing (IncomingMsgKind(..), kindDecoder, payload)

import Json.Decode exposing (field, succeed, fail)
import Dict exposing (Dict)


type IncomingMsgKind = Default
    | RegisterSuccess
    | Play
    | Pause
    | Resume
    | Stop
    | FsChange
    | FsState

type IncomingMsgPayload = FsStatePayload (Dict String String) | NoPayload

type alias MediaItem =
  {
    id: Int
  , name: String
  }


payload : IncomingMsgKind -> Json.Decode.Decoder IncomingMsgPayload
payload kind =
  case kind of
    FsState -> Json.Decode.map FsStatePayload (field "media" (Json.Decode.dict Json.Decode.string))
    _ -> fail <| "No payload can be decoded for this msg kind"

-- MediaItem decoding
decodeMediaItem : Json.Decode.Decoder MediaItem
decodeMediaItem =
    Json.Decode.map2 MediaItem
        (field "id" Json.Decode.int)
        (field "name" Json.Decode.string)

-- MsgKind decoding
kindDecoder : Json.Decode.Decoder IncomingMsgKind
kindDecoder = field "type" (Json.Decode.string |> Json.Decode.andThen kindDecoderFromString)

kindDecoderFromString : String -> Json.Decode.Decoder IncomingMsgKind
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