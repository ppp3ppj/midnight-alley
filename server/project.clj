(defproject midnight-alley "0.1.0-SNAPSHOT"
  :description "Midnight Alley server: a rat colony simulated with refs, agents, an atom and core.async"
  :dependencies [[org.clojure/clojure "1.11.3"]
                 [org.clojure/core.async "1.6.681"]
                 [http-kit "2.8.0"]
                 [cheshire "5.13.0"]]
  :main midnight-alley.core
  :source-paths ["src"]
  :test-paths ["test"])
