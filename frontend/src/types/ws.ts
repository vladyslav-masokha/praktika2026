export interface WsOrderStatusMessage {
  event_type: string;
  order_id: number;
  status: string;
  message: string;
}	