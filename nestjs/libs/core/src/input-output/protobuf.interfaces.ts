export interface ProtobufMessageWithPosition<T> {
  position: number;
  message: T;
}
