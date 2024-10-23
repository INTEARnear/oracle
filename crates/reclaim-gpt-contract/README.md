# Permissionless producer that uses Reclaim Protocol to validate responses

This example is a *smart contract producer* that stores incoming requests, and validates responses. But as opposed to a simple *account producer*, anyone can submit a response, and if it's legitimate, pass the response to the caller, and can be modified to send the fee to the account that called `submit` with the right data.

It validates not only the signature of the body, but also the response format, url, and some of the fields.

When deploying, don't forget to enable `set_send_callback`.
