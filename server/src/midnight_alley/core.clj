(ns midnight-alley.core
  (:require [midnight-alley.engine :as engine]
            [midnight-alley.server :as server]
            [midnight-alley.sim :as sim]
            [midnight-alley.world :as w])
  (:gen-class))

(defn -main [& _]
  (let [port (Integer/parseInt (or (System/getenv "PORT") "8080"))
        world (w/new-world)
        _ (sim/spawn-rats! world w/initial-rats)
        public-dir (or (System/getenv "PUBLIC_DIR") "public")
        eng (engine/start! world)]
    (server/start! eng port public-dir)
    (println (str "Midnight Alley: open http://localhost:" port "  (websocket on /ws, files from " public-dir ")"))))
