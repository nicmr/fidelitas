import Browser
import Html exposing (Html, button, div, text)
import Html.Events exposing (onClick)
import Http
import Url.Builder exposing (absolute)

main =
  Browser.element
  { init = init
  , update = update
  , view = view 
  , subscriptions = subscriptions
  }


-- Msg

type Msg = Increment | Decrement | Play  | Pause | Resume | Stop | DataReceived (Result Http.Error String)


-- Model 

type alias Model =
  { counter: Int
  , player_state: PlayerState
  }

type PlayerState = Paused

init : () -> (Model, Cmd Msg)
init _ =
  (
    { counter = 0
    , player_state = Paused
    }
    , Cmd.none
  )


-- Update

update : Msg -> Model -> (Model, Cmd Msg)
update msg model =
  case msg of
    Increment ->
      ( { model | counter = model.counter + 1}, Cmd.none )
    Decrement ->
      ( { model | counter = model.counter - 1}, Cmd.none )
    Play ->
      ( model, Http.get { url = absolute ["api", "play", "something"] [], expect = Http.expectString DataReceived})
    DataReceived result ->
      ( model, Cmd.none )
    Pause ->
      ( model, Http.get { url = absolute ["api", "pause"] [], expect = Http.expectString DataReceived})
    Resume ->
      ( model, Http.get { url = absolute ["api", "resume"] [], expect = Http.expectString DataReceived})
    Stop ->
      ( model, Http.get { url = absolute ["api", "stop"] [], expect = Http.expectString DataReceived})



-- Subscriptions

subscriptions : Model -> Sub Msg
subscriptions model =
  Sub.none


-- View
view : Model -> Html Msg
view model =
  div []
    [ button [ onClick Decrement ] [ text "-" ]
    , div [] [ text (String.fromInt model.counter) ]
    , button [ onClick Increment ] [ text "+" ]
    , button [ onClick Play] [text "Play"]
    , button [ onClick Pause] [text "Pause"]
    , button [ onClick Resume] [text "Resume"]
    , button [ onClick Stop] [text "Stop"]
    ]
