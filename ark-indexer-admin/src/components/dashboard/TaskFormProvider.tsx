"use client";

import React, {
  createContext,
  useContext,
  useState,
  type ReactNode,
} from "react";

export type TaskFormState = {
  from: string;
  to: string;
  numberOfTasks: string;
};

interface TaskFormContextType {
  values: TaskFormState;
  setValues: React.Dispatch<React.SetStateAction<TaskFormState>>;
}

const TaskFormContext = createContext<TaskFormContextType | undefined>(
  undefined,
);

interface TaskFormProviderProps {
  children: ReactNode;
}

export const TaskFormProvider: React.FC<TaskFormProviderProps> = ({
  children,
}) => {
  const [values, setValues] = useState<TaskFormState>({
    from: "0",
    to: "1000",
    numberOfTasks: "3",
  });

  return (
    <TaskFormContext.Provider value={{ values, setValues }}>
      {children}
    </TaskFormContext.Provider>
  );
};

export const useTaskForm = (): TaskFormContextType => {
  const context = useContext(TaskFormContext);

  if (!context) {
    throw new Error("useTaskForm must be used within a TaskFormProvider");
  }

  return context;
};
