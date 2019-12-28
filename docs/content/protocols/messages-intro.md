The following documents describe the communication between the fidelitas server and a client.
This will come in useful when for developers who would like to implement their own client for fidelitas.

## Message format
The messages are encoded as JSON objects.
We're considering switching to protobuf in the future. Progress on protobuf implementation can be checked at this issue: #XX.


## Recurring fields
Every message will contain a type field.
```json
{
    "type": ""
}
```

The messages will have different data fields depending on the type of message the server is transmitting.
We recommend decoding messages into sum types (enums), if your language offers support for them.
Alternatively, you will have to manually for the (string) value of the type field to determine the kind of message you received.

The next chapters will describe the specific messages and data fields the server will send and accept.