# surge

Asynchronous server for collecting offline rollouts in a [reinforcement learning](https://en.wikipedia.org/wiki/Reinforcement_learning#Introduction) setting

1. Externally, Pytorch models of agent policy functions are are trained using PPO
2. Models weights are are sent by clients to be cached in the server
3. Each model version plays multiple matches against all other models
4. Rollouts of these matches are collected and returned to the clients

A fruitbots clone is used as the game environment in this engine
